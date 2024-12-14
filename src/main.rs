//! Reverse proxy listening in "localhost:4000" will proxy all requests to "localhost:3000"
//! endpoint.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -p example-reverse-proxy
//! ```

use axum::{
    body::Body,
    extract::{Request, State},
    http::uri::Uri,
    response::Response,
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use hyper::StatusCode;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    let client: Client =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());

    let app = Router::new().route("/", get(handler)).with_state(client);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(
    State(client): State<Client>,
    mut req: Request,
) -> Result<axum::response::Response, StatusCode> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://127.0.0.1:3000{}", path_query);

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    let upstream_response = client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let (head, body) = upstream_response.into_parts();

    let body = body.collect().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let body = body.to_bytes();

    let response = Response::from_parts(head, Body::from(body));

    Ok(response)
}
