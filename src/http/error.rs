use thiserror::Error;

#[derive(Debug, Error, strum::IntoStaticStr)]
pub enum Upstream {
    #[error("error reading upstream response body")]
    ReadBody(hyper::Error),

    #[error("error sending request to upstream: {0}")]
    Request(#[source] hyper_util::client::legacy::Error),

    #[error(transparent)]
    InvalidUri(#[from] http::uri::InvalidUri),
}
