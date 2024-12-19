use crate::error::recover;
use crate::proxy::Proxy;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use std::sync::Arc;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::error;

mod error;
mod log;
mod proxy;
mod representation;
mod sample;
mod slurp;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::init();

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let proxy = Arc::new(Proxy::new(
        "http://127.0.0.1:3001".into(),
        "http://127.0.0.1:3000".into(),
        &["/api/{value}"],
    ));

    let trace_layer = log::new_trace_layer();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let proxy = proxy.clone();

        let svc = ServiceBuilder::new()
            .layer(trace_layer.clone())
            .service_fn(move |request| {
                let proxy = proxy.clone();
                async move { proxy.handle(request).await.or_else(recover) }
            });
        let svc = TowerToHyperService::new(svc);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                error!("Error serving connection: {err:?}");
            }
        });
    }
}
