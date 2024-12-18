use crate::slurp::{slurp_request, slurp_response};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use rdkafka::producer::FutureRecord;
use rdkafka::ClientConfig;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;

#[derive(Clone)]
pub struct Proxy {
    client: Client<HttpConnector, Full<Bytes>>,
    candidate_base: String,
    reference_base: String,
    router: matchit::Router<bool>,
    producer: rdkafka::producer::FutureProducer,
    topic: String,
}

impl Proxy {
    pub(crate) fn new(candidate_base: String, reference_base: String, routes: &[&str]) -> Self {
        let mut router = matchit::Router::new();

        for &r in routes {
            router.insert(r, true).unwrap();
        }

        let producer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "5000")
            .create()
            .unwrap();

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

    /// send the query to the candidate
    pub async fn query_candidate(&self, req: Request<Full<Bytes>>) -> Response<Bytes> {
        let candidate_response = self
            .client
            .request(req)
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)
            .unwrap();

        slurp_response(candidate_response).await.unwrap()
    }

    /// mirror the request to the candidate
    ///
    /// receives the reference-response via the returned sender
    pub fn mirror(
        &self,
        mut req: Request<Full<Bytes>>,
        path_query: &str,
    ) -> Sender<Response<Bytes>> {
        let (tx, rx) = oneshot::channel::<Response<Bytes>>();

        let candidate_uri = format!("{}{}", self.candidate_base, path_query);
        *req.uri_mut() = Uri::try_from(candidate_uri).unwrap();

        let self_clone = self.clone();
        tokio::spawn(async move {
            let response = self_clone.query_candidate(req).await;
            let reference = rx.await.unwrap();

            let (candidate_header, candidate_body) = response.into_parts();
            let (reference_header, reference_body) = reference.into_parts();

            let message = format!("Candidate: {candidate_header:?} {candidate_body:#?}\nReference: {reference_header:?} {reference_body:#?}");

            let delivery_status = self_clone
                .producer
                .send::<(), _, _>(
                    FutureRecord::to(&self_clone.topic).payload(&message),
                    Duration::from_secs(0),
                )
                .await;

            println!("{delivery_status:?}")
        });

        tx
    }

    pub async fn handle(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> hyper::Result<Response<Full<Bytes>>> {
        let is_shadow_test = self.router.at(req.uri().path()).is_ok_and(|it| *it.value);

        let mut req = slurp_request(req).await.unwrap();

        let path = req.uri().path();
        let path_query = req
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or(path);

        let tx = if is_shadow_test {
            Some(self.mirror(req.clone(), path_query))
        } else {
            None
        };

        let reference_uri = format!("{}{}", self.reference_base, path_query);
        *req.uri_mut() = Uri::try_from(reference_uri).unwrap();

        let response = self
            .client
            .request(req)
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)
            .unwrap();

        let response = slurp_response(response).await.unwrap();

        // send the reference-response over to the candidate-task
        if let Some(tx) = tx {
            tx.send(response.clone()).unwrap();
        }

        Ok(response.map(Full::new))
    }
}
