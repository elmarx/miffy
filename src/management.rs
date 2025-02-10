use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::error;

pub async fn run(management_port: u16) -> tokio::io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], management_port));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let svc = ServiceBuilder::new().service_fn(move |request: Request<Incoming>| async move {
            let response = match (request.method(), request.uri().path()) {
                (&Method::GET, "/healthz") => {
                    Response::new(Full::new(Bytes::from(r#"{"status": "healthy"}"#)))
                }
                _ => {
                    let mut not_found = Response::new(Full::new(Bytes::new()));
                    *not_found.status_mut() = StatusCode::NOT_FOUND;
                    not_found
                }
            };

            Result::<_, Infallible>::Ok(response)
        });
        let svc = TowerToHyperService::new(svc);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                error!("Error serving connection: {err:?}");
            }
        });
    }
}
