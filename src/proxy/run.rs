use crate::proxy;
use crate::proxy::error::recover;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::error;

/// run the main server loop
pub async fn run(proxy: proxy::Service) -> tokio::io::Result<()> {
    let proxy = Arc::new(proxy);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    let trace_layer = proxy::log::new_trace_layer();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let proxy = proxy.clone();

        let svc = ServiceBuilder::new()
            .layer(trace_layer.clone())
            .service_fn(move |request| {
                let proxy = proxy.clone();
                async move {
                    match crate::http::slurp::request(request).await {
                        Ok(request) => proxy.handle(request).await.or_else(recover),
                        Err(e) => proxy::error::handle_incoming_request(&e),
                    }
                }
            });
        let svc = TowerToHyperService::new(svc);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                error!("Error serving connection: {err:?}");
            }
        });
    }
}
