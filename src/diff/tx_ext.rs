use crate::domain;
use crate::http::error;
use bytes::Bytes;
use http::Response;
use tokio::sync::oneshot::Sender;
use tracing::error;

pub trait TxExt {
    /// send the reference response over to the mirror-task
    ///
    /// the sender may be None, then nothing will be done.
    fn send_reference(self, response: &Result<Response<Bytes>, error::Upstream>);
}

impl TxExt for Option<Sender<domain::Result>> {
    fn send_reference(self, response: &Result<Response<Bytes>, error::Upstream>) {
        if let Some(tx) = self {
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
    }
}
