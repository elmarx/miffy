use super::util::header_ext::TxHeader;
use crate::util::serialization;
use bytes::Bytes;
use serde::Serialize;
use serde_json::Value;
use serde_with::base64::Base64;
use serde_with::serde_as;
use std::collections::HashMap;

/// a simplified representation of technical errors that may be cloned, serialized etc.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Error {
    Uri,
    Request,
    Body,
}

impl From<&crate::http::error::Upstream> for Error {
    fn from(value: &crate::http::error::Upstream) -> Self {
        match value {
            crate::http::error::Upstream::InvalidUri(_) => Error::Uri,
            crate::http::error::Upstream::Request(_) => Error::Request,
            crate::http::error::Upstream::ReadBody(_) => Error::Body,
        }
    }
}

/// sample represents a shadow-tested request, i.e. a mirrored request that may be analyzed further
#[derive(Serialize)]
pub struct Sample {
    pub request: Request,
    pub reference: RequestResult,
    pub candidate: RequestResult,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RequestResult {
    pub url: String,
    #[serde(serialize_with = "serialization::custom_result")]
    pub response: Result<Response, Error>,
}

impl RequestResult {
    pub fn new(url: String, response: Result<Response, Error>) -> Self {
        Self { url, response }
    }
}

impl Sample {
    pub(crate) fn new(
        request: Request,
        reference: RequestResult,
        candidate: RequestResult,
    ) -> Self {
        Self {
            request,
            reference,
            candidate,
        }
    }

    pub fn is_equal(&self) -> bool {
        match (&self.reference.response, &self.candidate.response) {
            // TODO: maybe compare a relevant subset of headers, e.g. "Location"
            (Ok(a), Ok(b)) => a.status == b.status && a.body == b.body,
            // if any of these fail, they are obviously different. If both fail that's strange, and we're going to report this
            _ => false,
        }
    }
}

#[serde_as]
#[derive(Serialize, Debug, PartialEq)]
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
        } else if headers
            .get(http::header::CONTENT_TYPE)
            .is_some_and(TxHeader::is_json)
            || headers
                .get(http::header::ACCEPT)
                .is_some_and(TxHeader::is_json)
        {
            serde_json::from_slice(bytes)
                .map(Self::Json)
                .unwrap_or(Self::Bytes(bytes.clone()))
        } else {
            Self::Bytes(bytes.clone())
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
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
    pub method: http::Method,
    #[serde(with = "http_serde::uri")]
    pub uri: http::Uri,
    pub route: String,
    pub params: HashMap<String, String>,
    pub body: Body,
}

impl Request {
    pub fn new(
        request: http::Request<Bytes>,
        route: String,
        route_params: Vec<(String, String)>,
    ) -> Self {
        let body = Body::new(request.headers(), request.body());

        Self {
            method: request.method().clone(),
            uri: request.uri().clone(),
            route,
            params: route_params.into_iter().collect(),
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
    use super::{Body, Request, RequestResult, Response};
    use bytes::Bytes;
    use http::{HeaderMap, HeaderValue};

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

    #[test]
    fn test_do_not_compare_headers() {
        let mut headers_a = HeaderMap::new();
        headers_a.append("date", HeaderValue::from_static("now"));

        let mut headers_b = HeaderMap::new();
        headers_b.append(
            "date",
            HeaderValue::from_static("now plus a little bit later"),
        );

        let sample = super::Sample::new(
            Request {
                method: http::Method::GET,
                uri: "http://localhost".parse().unwrap(),
                route: "path".to_string(),
                params: Default::default(),
                body: Body::None,
            },
            RequestResult::new(
                "http://localhost:3000".to_string(),
                Ok(Response {
                    status: http::StatusCode::OK,
                    headers: headers_a,
                    body: Body::Json(serde_json::json!({"c:": "d", "a": "b"})),
                }),
            ),
            RequestResult::new(
                "http://localhost:3001".to_string(),
                Ok(Response {
                    status: http::StatusCode::OK,
                    headers: headers_b,
                    body: Body::Json(serde_json::json!({"a": "b", "c:": "d"})),
                }),
            ),
        );

        assert!(sample.is_equal());
    }

    #[test]
    fn test_do_compare_status_code() {
        let sample = super::Sample::new(
            Request {
                method: http::Method::GET,
                uri: "http://localhost".parse().unwrap(),
                route: "path".to_string(),
                params: Default::default(),
                body: Body::None,
            },
            RequestResult::new(
                "http://localhost:3000".to_string(),
                Ok(Response {
                    status: http::StatusCode::OK,
                    headers: Default::default(),
                    body: Body::Json(serde_json::json!({"c:": "d", "a": "b"})),
                }),
            ),
            RequestResult::new(
                "http://localhost:3000".to_string(),
                Ok(Response {
                    status: http::StatusCode::BAD_REQUEST,
                    headers: Default::default(),
                    body: Body::Json(serde_json::json!({"a": "b", "c:": "d"})),
                }),
            ),
        );

        assert!(!sample.is_equal());
    }
}
