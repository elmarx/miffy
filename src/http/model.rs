use crate::domain::RequestResult;
use bytes::Bytes;
use tokio::sync::oneshot::{Receiver, Sender};

/// a mode for this request.
///
/// Either it's an experiment, or simple proxy
pub enum RequestMode {
    Proxy,
    Experiment {
        request: http::Request<Bytes>,
        candidate_uri: String,
        rx: Receiver<RequestResult>,
    },
}

/// context for a request: the (live/reference) upstream uri to use, the mode, and an optinioal sender to send results to
pub struct RequestContext {
    pub reference_uri: String,
    pub tx: Option<Sender<RequestResult>>,
    pub mode: RequestMode,
}
