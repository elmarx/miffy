use crate::domain;
use crate::model::Filter;
use bytes::Bytes;
use http::Response;
use std::sync::Arc;
use tokio::sync::oneshot::{Receiver, Sender};

/// type of the value sent over the channel
pub type ChannelValue = (String, Result<Response<Bytes>, domain::Error>);

/// a mode for this request.
///
/// Either it's an experiment, or simple proxy
pub enum RequestMode {
    Proxy,
    Experiment(Experiment),
}

/// parameters to mirror the request, i.e. conduct the experiment
pub struct Experiment {
    /// if given in config: custom key
    pub key: Option<String>,
    /// path of the route
    pub route: String,
    /// parameters as extracted from the route
    pub route_params: Vec<(String, String)>,
    pub original_request: http::Request<Bytes>,
    pub candidate_uri: String,
    pub rx: Receiver<ChannelValue>,
    #[expect(dead_code)]
    pub reference_filter: Option<Arc<Filter>>,
    #[expect(dead_code)]
    pub candidate_filter: Option<Arc<Filter>>,
}

/// context for a request: the (live/reference) upstream uri to use, the mode, and an optional sender to send results to
pub struct RequestContext {
    pub reference_uri: String,
    pub tx: Option<Sender<ChannelValue>>,
    pub mode: RequestMode,
}
