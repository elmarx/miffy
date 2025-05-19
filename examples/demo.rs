use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::TypedHeader;
use serde_json::json;
use std::error::Error;
use std::str::FromStr;
use tracing::info;

use headers::{Header, HeaderName, HeaderValue};
use strum::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
enum ShadowTestRole {
    #[strum(ascii_case_insensitive)]
    Reference,
    #[strum(ascii_case_insensitive)]
    Candidate,
    #[strum(ascii_case_insensitive)]
    Upstream,
}
static SHADOW_TEST_HEADER: HeaderName = HeaderName::from_static("x-shadow-test-role");

impl Header for ShadowTestRole {
    fn name() -> &'static HeaderName {
        &SHADOW_TEST_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let value = value.to_str().map_err(|_| headers::Error::invalid())?;

        ShadowTestRole::from_str(value).map_err(|_| headers::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let s = self.as_ref();
        let value = HeaderValue::from_str(s).expect("static value should never panic");

        values.extend(std::iter::once(value));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app1 = Router::new().route(
        "/api/{value}",
        get(|Path(value): Path<i32>, role: Option<TypedHeader<ShadowTestRole>>| async move {
            if let Some(TypedHeader(role)) = role {
                Json(json!({ "msg": format!("I am the {}", role.as_ref()), "result": value + 1 }))
            } else {
                Json(json!({ "msg": "I'm not part of a test", "result": value + 1 }))
            }
        }),
    );

    let app2 = Router::new().route(
        "/api/{value}",
        get(
            |Path(value): Path<i32>, role: Option<TypedHeader<ShadowTestRole>>| async move {
                if let Some(TypedHeader(role)) = role {
                    Json(json!({ "msg": format!("I am the {}", role.as_ref()), "result": value + 100 }))
                } else {
                    Json(json!({ "msg": "I'm not part of a test", "result": value + 100 }))
                }
            },
        ),
    );

    let listener1 = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let listener2 = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;

    info!(
        "listening on {} and {}",
        listener1.local_addr()?,
        listener2.local_addr()?
    );

    tokio::spawn(async move { axum::serve(listener1, app1).await });
    axum::serve(listener2, app2).await?;

    Ok(())
}
