use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use std::error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app1 = Router::new().route(
        "/api/{value}",
        get(|Path(value): Path<i32>| async move {
            Json(json!({ "msg": "A message", "result": value + 1 }))
        }),
    );

    let app2 = Router::new().route(
        "/api/{value}",
        get(|Path(value): Path<i32>| async move {
            Json(json!({ "msg": "Another message", "result": value + 100 }))
        }),
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
