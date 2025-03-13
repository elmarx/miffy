use crate::domain;
use bytes::Bytes;
use http::Response;
use tokio::sync::oneshot::{Receiver, Sender};

/// type of the value sent over the channel
pub type ChannelValue = (String, Result<Response<Bytes>, domain::Error>);

/// a mode for this request.
///
/// Either it's an experiment, or simple proxy
pub enum RequestMode {
    Proxy,
    Experiment {
        /// if given in config: custom key
        key: Option<String>,
        /// path of the route
        route: String,
        /// parameters as extracted from the route
        route_params: Vec<(String, String)>,
        request: http::Request<Bytes>,
        candidate_uri: String,
        rx: Receiver<ChannelValue>,
    },
}

/// context for a request: the (live/reference) upstream uri to use, the mode, and an optional sender to send results to
pub struct RequestContext {
    pub reference_uri: String,
    pub tx: Option<Sender<ChannelValue>>,
    pub mode: RequestMode,
}
