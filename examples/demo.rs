use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { Json(json!({ "msg": "Hello World" })) }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
