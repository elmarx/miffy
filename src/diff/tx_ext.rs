use crate::http::error;
use crate::http::model::ChannelValue;
use bytes::Bytes;
use http::Response;
use tokio::sync::oneshot::Sender;
use tracing::error;

pub trait TxExt {
    /// send the reference response over to the mirror-task
    ///
    /// the sender may be None, then nothing will be done.
    fn send_reference(self, url: String, response: &Result<Response<Bytes>, error::Upstream>);
}

impl TxExt for Option<Sender<ChannelValue>> {
    fn send_reference(self, url: String, response: &Result<Response<Bytes>, error::Upstream>) {
        if let Some(tx) = self {
            let response = match response {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(e.into()),
            };

            if let Err(e) = tx.send((url, response)) {
                // sending over the response failed, that's a shame, but it just means testing failed, we can still successfully respond to the client
                error!("error sending response to shadow-test: {e:?}");
            }
        }
    }
}
