use crate::proxy;
use crate::proxy::error::recover;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::error;

/// run the main server loop
pub async fn run(port: u16, proxy: proxy::Service) -> tokio::io::Result<()> {
    let proxy = Arc::new(proxy);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // Spawn a thread for each available core, minus one, since we'll
    // reuse the main thread as a server thread as well.
    for _ in 1..num_cpus::get() {
        let proxy = proxy.clone();
        let addr = addr.clone();
        thread::spawn(move || async move {
            server_thread(addr, proxy).await.unwrap();
        });
    }

    server_thread(addr, proxy).await
}

pub async fn server_thread(addr: SocketAddr, proxy: Arc<proxy::Service>) -> tokio::io::Result<()> {
    // let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = reuse_listener(&addr)?;

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

fn reuse_listener(addr: &SocketAddr) -> std::io::Result<TcpListener> {
    let builder = match *addr {
        SocketAddr::V4(_) => net2::TcpBuilder::new_v4()?,
        SocketAddr::V6(_) => net2::TcpBuilder::new_v6()?,
    };

    #[cfg(unix)]
    {
        use net2::unix::UnixTcpBuilderExt;
        if let Err(e) = builder.reuse_port(true) {
            error!("error setting SO_REUSEPORT: {}", e);
        }
    }

    builder.reuse_address(true)?;
    builder.bind(addr)?;
    let listener = builder.listen(1024)?;
    listener.set_nonblocking(true)?;
    TcpListener::from_std(listener)
}
