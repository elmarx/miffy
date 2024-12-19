use crate::sample;
use bytes::Bytes;
use http::{HeaderValue, Request, Response};
use serde::Serialize;
use serde_json::Value;
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct ResponseRepr {
    #[serde(with = "http_serde::status_code")]
    status: http::StatusCode,
    #[serde(with = "http_serde::header_map")]
    headers: http::HeaderMap,
    body: Body,
}

#[derive(Serialize)]
pub struct RequestRepr {
    #[serde(with = "http_serde::method")]
    method: http::Method,
    #[serde(with = "http_serde::uri")]
    uri: http::Uri,
    body: Body,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ResultRepr {
    Ok(ResponseRepr),
    Err { error: sample::Error },
}

impl From<sample::Result> for ResultRepr {
    fn from(value: sample::Result) -> Self {
        match value {
            Ok(response) => ResultRepr::Ok(response.into()),
            Err(error) => ResultRepr::Err { error },
        }
    }
}

impl From<Response<Bytes>> for ResponseRepr {
    fn from(value: Response<Bytes>) -> Self {
        let body = Body::new(value.headers(), value.body());

        Self {
            status: value.status(),
            headers: value.headers().clone(),
            body,
        }
    }
}

impl From<Request<Bytes>> for RequestRepr {
    fn from(value: Request<Bytes>) -> Self {
        let body = Body::new(value.headers(), value.body());

        Self {
            method: value.method().clone(),
            uri: value.uri().clone(),
            body,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use crate::representation::Body;
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
