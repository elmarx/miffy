use crate::serialization;
use bytes::Bytes;
use http::HeaderValue;
use serde::Serialize;
use serde_json::Value;
use serde_with::base64::Base64;
use serde_with::serde_as;

pub type Result = std::result::Result<Response, Error>;

/// a simplified representation of technical errors that may be cloned, serialized etc.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Error {
    Uri,
    Request,
    Body,
}

impl From<&crate::proxy::error::Upstream> for Error {
    fn from(value: &crate::proxy::error::Upstream) -> Self {
        match value {
            crate::proxy::error::Upstream::InvalidUri(_) => Error::Uri,
            crate::proxy::error::Upstream::Request(_) => Error::Request,
            crate::proxy::error::Upstream::ReadBody(_) => Error::Body,
        }
    }
}

/// sample represents a shadow-tested request, i.e. a mirrored request that may be analyzed further
#[derive(Serialize)]
pub struct Sample {
    request: Request,
    #[serde(serialize_with = "serialization::custom_result")]
    reference: Result,
    #[serde(serialize_with = "serialization::custom_result")]
    candidate: Result,
}

impl Sample {
    pub(crate) fn new(request: Request, reference: Result, candidate: Result) -> Self {
        Self {
            request,
            reference,
            candidate,
        }
    }
}

#[serde_as]
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "lowercase")]
pub enum Body {
    Bytes(#[serde_as(as = "Base64")] Bytes),
    Json(Value),
    #[serde(untagged)]
    None,
}

impl Body {
    fn new(headers: &http::HeaderMap, bytes: &Bytes) -> Self {
        if bytes.is_empty() {
            Self::None
        } else if headers.get(http::header::CONTENT_TYPE)
            == Some(&HeaderValue::from_static("application/json"))
        {
            serde_json::from_slice(bytes)
                .map(Self::Json)
                .unwrap_or(Self::Bytes(bytes.clone()))
        } else {
            Self::Bytes(bytes.clone())
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Response {
    #[serde(with = "http_serde::status_code")]
    status: http::StatusCode,
    #[serde(with = "http_serde::header_map")]
    headers: http::HeaderMap,
    body: Body,
}

#[derive(Serialize)]
pub struct Request {
    #[serde(with = "http_serde::method")]
    method: http::Method,
    #[serde(with = "http_serde::uri")]
    uri: http::Uri,
    body: Body,
}

impl From<http::Request<Bytes>> for Request {
    fn from(value: http::Request<Bytes>) -> Self {
        let body = Body::new(value.headers(), value.body());

        Self {
            method: value.method().clone(),
            uri: value.uri().clone(),
            body,
        }
    }
}

impl From<http::Response<Bytes>> for Response {
    fn from(value: http::Response<Bytes>) -> Self {
        let body = Body::new(value.headers(), value.body());

        Self {
            status: value.status(),
            headers: value.headers().clone(),
            body,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use super::Body;
    use bytes::Bytes;

    #[test]
    fn serialize_body_none() {
        let sample = Body::None;

        let actual = serde_json::to_string(&sample).unwrap();

        assert_eq!(actual, "null");
    }

    #[test]
    fn serialize_body_value() {
        let sample = Body::Json(serde_json::json!({"a":"b"}));

        let actual = serde_json::to_string(&sample).unwrap();

        assert_eq!(actual, r#"{"type":"json","value":{"a":"b"}}"#);
    }

    #[test]
    fn serialize_body_bytes() {
        let b = Bytes::from_static(&[1, 2, 3]);

        let sample = Body::Bytes(b);

        let actual = serde_json::to_string(&sample).unwrap();

        assert_eq!(actual, r#"{"type":"bytes","value":"AQID"}"#);
    }
}
