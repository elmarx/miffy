use crate::error;
use bytes::Bytes;
use http::{Response, StatusCode};
use http_body_util::Full;
use serde_json::json;
use std::convert::Infallible;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

pub type Result<T> = std::result::Result<T, Miffy>;

#[derive(Debug, Error, strum::IntoStaticStr)]
pub enum Miffy {
    #[error("error reading request body")]
    ReadRequestBody(hyper::Error),

    #[error("error reading upstream response body")]
    ReadResponseBody(hyper::Error),

    #[error("error sending request to upstream: {0}")]
    UpstreamRequest(#[source] hyper_util::client::legacy::Error),

    #[error(transparent)]
    InvalidUri(#[from] http::uri::InvalidUri),
}

#[derive(Debug, Error)]
pub enum Internal {
    #[error(transparent)]
    RecvError(#[from] RecvError),

    #[error(transparent)]
    Miffy(#[from] Miffy),
}

/// recover from errors by providing an error-response
pub fn recover(err: Miffy) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    Ok(err.into())
}

impl From<Miffy> for Response<Full<Bytes>> {
    fn from(value: error::Miffy) -> Self {
        let status = match value {
            Miffy::ReadRequestBody(_) => StatusCode::BAD_REQUEST,
            Miffy::ReadResponseBody(_) => StatusCode::BAD_GATEWAY,
            Miffy::InvalidUri(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Miffy::UpstreamRequest(_) => StatusCode::BAD_GATEWAY,
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
