use crate::diff::error::Internal;
use crate::diff::publisher::Publisher;
use crate::domain;
use crate::domain::Sample;
use crate::http::client::{Client, UpstreamExt};
use crate::http::model::{Experiment, RequestMode};
use tracing::error;

pub fn build_key(key: String, params: &[(String, String)]) -> String {
    params
        .iter()
        .find(|(k, _)| k == &key)
        .map(|(_, v)| v.to_string())
        .unwrap_or(key)
}

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
            .build_http(),
            publisher,
        }
    }

    /// mirror the original request to the candidate and wait for the reference
    pub async fn mirror(&self, xp: Experiment) -> Result<(), Internal> {
        let response = self
            .client
            .upstream(xp.original_request.clone(), &xp.candidate_uri)
            .await;

        // if the sender is dropped, this will receive a RecvError, we're just logging an error then
        let (reference_uri, reference_res) = xp.rx.await?;
        let reference = domain::RequestResult::new(reference_uri, reference_res.map(Into::into));

        let response = response.map(Into::into).map_err(|e| (&e).into());
        let response = domain::RequestResult::new(xp.candidate_uri, response);

        let key = xp.key.map_or_else(
            || xp.route.clone(),
            |key| build_key(key, xp.route_params.as_slice()),
        );

        // once we have the response of the reference and the candidate, let the publisher process this sample
        let sample = Sample::new(
            domain::Request::new(&xp.original_request, xp.route, xp.route_params),
            reference,
            response,
        );
        self.publisher.publish(&key, sample).await;

        Ok(())
    }

    /// spawn a mirror-task based on the given mode
    pub fn spawn(&self, mode: RequestMode) {
        match mode {
            RequestMode::Proxy => {}
            RequestMode::Experiment(xp) => {
                let self_clone = self.clone();
                tokio::spawn(async move {
                    // if this fails it just means the mirroring failed (for any reason). The actual request (to the reference) is not impacted
                    if let Err(e) = self_clone.mirror(xp).await {
                        error!("internal error mirroring request: {e:?}.");
                    }
                });
            }
        }
    }
}
