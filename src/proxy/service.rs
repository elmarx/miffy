use crate::domain::Sample;
use crate::proxy::error::{Internal, Upstream};
use crate::proxy::slurp;
use crate::{domain, error};
use http::uri::PathAndQuery;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use rdkafka::producer::FutureRecord;
use rdkafka::ClientConfig;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use tracing::debug;

type Result<T> = std::result::Result<T, Upstream>;

#[derive(Clone)]
pub struct Service {
    client: Client<HttpConnector, Full<Bytes>>,
    candidate_base: String,
    reference_base: String,
    router: matchit::Router<bool>,
    producer: rdkafka::producer::FutureProducer,
    topic: String,
}

impl Service {
    pub(crate) fn new(candidate_base: String, reference_base: String, routes: &[&str]) -> Self {
        let mut router = matchit::Router::new();

        for &r in routes {
            router.insert(r, true).expect("invalid path provided");
        }

        let producer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("invalid kafka configuration");

        Self {
            client: Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(HttpConnector::new()),
            candidate_base,
            reference_base,
            router,
            producer,
            topic: "miffy".to_string(),
        }
    }

    pub async fn request(&self, mut req: Request<Bytes>, uri: String) -> Result<Response<Bytes>> {
        *req.uri_mut() = Uri::try_from(uri)?;

        let response = self
            .client
            .request(req.map(Full::new))
            .await
            .map_err(Upstream::Request)?;

        slurp::response(response).await
    }

    pub async fn publish(&self, sample: Sample) {
        let message = serde_json::to_string(&sample).expect("failed to serialize sample-message");

        let delivery_status = self
            .producer
            .send::<(), _, _>(
                FutureRecord::to(&self.topic).payload(&message),
                Duration::from_secs(0),
            )
            .await;
        debug!("Delivery status: {delivery_status:?}");
    }

    pub async fn mirror(
        &self,
        original_request: Request<Bytes>,
        candidate_uri: String,
        rx: Receiver<domain::Result>,
    ) -> std::result::Result<(), Internal> {
        let response = self.request(original_request.clone(), candidate_uri).await;

        // if the sender is dropped, this will receive a RecvError, we're just logging an error then
        let reference = rx.await?;

        let response: domain::Result = response.map(Into::into).map_err(|e| (&e).into());

        let sample = Sample::new(original_request.into(), reference, response);
        self.publish(sample).await;

        Ok(())
    }

    /// spawn an independent task that mirrors the request to the client and publishes the results
    ///
    /// returns a sender for the main-thread to send over the actual response by the reference
    pub fn spawn_mirror(&self, req: Request<Bytes>, path_query: &str) -> Sender<domain::Result> {
        let candidate_uri = format!("{}{}", self.candidate_base, path_query);

        let (tx, rx) = oneshot::channel::<domain::Result>();
        let self_clone = self.clone();
        tokio::spawn(async move {
            // if this fails it just means the mirroring failed (for any reason). The actual request (to the reference) is not impacted
            if let Err(e) = self_clone.mirror(req, candidate_uri, rx).await {
                error!("internal error mirroring request: {e:?}.");
            }
        });

        tx
    }

    pub async fn handle(&self, req: Request<Bytes>) -> Result<Response<Full<Bytes>>> {
        let is_shadow_test = self.router.at(req.uri().path()).is_ok_and(|it| *it.value);

        let path_query = req
            .uri()
            .path_and_query()
            .map_or(req.uri().path(), PathAndQuery::as_str);

        let tx = if is_shadow_test {
            Some(self.spawn_mirror(req.clone(), path_query))
        } else {
            None
        };

        let reference_uri = format!("{}{}", self.reference_base, path_query);

        let response = self.request(req, reference_uri).await;

        // send the reference-response over to the candidate-task
        if let Some(tx) = tx {
            // turn into a domain::Result before sending over for comparison
            let response = (response)
                .as_ref()
                .map(|r| r.clone().into())
                .map_err(Into::into);

            if let Err(e) = tx.send(response) {
                // sending over the response failed, that's a shame, but it just means testing failed, we can still successfully respond to the client
                error!("error sending response to shadow-test: {e:?}");
            }
        }

        response.map(|r| r.map(Full::new))
    }
}
