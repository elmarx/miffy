use crate::error;
use bytes::Bytes;
use http::{Response, StatusCode};
use http_body_util::Full;
use serde_json::json;
use std::convert::Infallible;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

pub type Result<T> = std::result::Result<T, Upstream>;

#[derive(Debug, Error, strum::IntoStaticStr)]
pub enum Upstream {
    #[error("error reading upstream response body")]
    ReadBody(hyper::Error),

    #[error("error sending request to upstream: {0}")]
    Request(#[source] hyper_util::client::legacy::Error),

    #[error(transparent)]
    InvalidUri(#[from] http::uri::InvalidUri),
}

#[derive(Debug, Error)]
pub enum Internal {
    #[error(transparent)]
    RecvError(#[from] RecvError),

    #[error(transparent)]
    Upstream(#[from] Upstream),
}

/// recover from errors by providing an error-response
#[expect(clippy::unnecessary_wraps)]
pub fn recover(err: Upstream) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    Ok(err.into())
}

/// generate a response if reading the incoming request to the proxy fails.
/// These are typically TCP-errors (where the client is already gone), so returning a response is probably
/// useless, but anyway
#[expect(clippy::unnecessary_wraps)]
pub fn handle_incoming_request(
    error: &hyper::Error,
) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(error.to_string().into())
        .expect("OK"))
}

impl From<Upstream> for Response<Full<Bytes>> {
    fn from(value: error::Upstream) -> Self {
        let status = match value {
            Upstream::ReadBody(_) | Upstream::Request(_) => StatusCode::BAD_GATEWAY,
            Upstream::InvalidUri(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error: &str = (&value).into();
        let message = value.to_string();

        let body = json!({ "error": error, "message": message });
        let body = serde_json::to_string(&body).expect("failed to serialize error-response-body");
        let body = body.into();

        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(body)
            .expect("failed to build error-response")
    }
}