use crate::diff::error::Internal;
use crate::diff::publisher::Publisher;
use crate::domain;
use crate::domain::Sample;
use crate::http::client::{Client, UpstreamExt};
use crate::http::model::RequestMode;
use bytes::Bytes;
use http::Request;
use hyper_util::client::legacy::connect::HttpConnector;
use tokio::sync::oneshot::Receiver;
use tracing::error;

/// a mirror will be initialized once per request
#[derive(Clone)]
pub struct Mirror {
    client: Client,
    publisher: Publisher,
}

impl Mirror {
    pub fn new(publisher: Publisher) -> Self {
        Self {
            client: hyper_util::client::legacy::Client::builder(
                hyper_util::rt::TokioExecutor::new(),
            )
            .build(HttpConnector::new()),
            publisher,
        }
    }

    /// mirror the original request to the candidate and wait for the reference
    pub async fn mirror(
        &self,
        original_request: Request<Bytes>,
        candidate_uri: String,
        reference_rx: Receiver<domain::RequestResult>,
    ) -> Result<(), Internal> {
        let response = self
            .client
            .upstream(original_request.clone(), &candidate_uri)
            .await;

        // if the sender is dropped, this will receive a RecvError, we're just logging an error then
        let reference = reference_rx.await?;

        let response = response.map(Into::into).map_err(|e| (&e).into());
        let response = domain::RequestResult::new(candidate_uri, response);

        // once we have the response of the reference and the candidate, let the publisher process this sample
        let sample = Sample::new(original_request.into(), reference, response);
        self.publisher.publish(sample).await;

        Ok(())
    }

    /// spawn a mirror-task based on the given mode
    pub fn spawn(&self, mode: RequestMode) {
        match mode {
            RequestMode::Proxy => {}
            RequestMode::Experiment {
                request,
                candidate_uri,
                rx,
            } => {
                let self_clone = self.clone();
                tokio::spawn(async move {
                    // if this fails it just means the mirroring failed (for any reason). The actual request (to the reference) is not impacted
                    if let Err(e) = self_clone.mirror(request, candidate_uri, rx).await {
                        error!("internal error mirroring request: {e:?}.");
                    }
                });
            }
        }
    }
}
