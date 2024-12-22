use diff::dispatcher::Dispatcher;
use diff::mirror::Mirror;
use diff::publisher::Publisher;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use util::log;

mod diff;
mod domain;
mod http;
mod proxy;
mod util;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    log::init();

    let publisher = Publisher::new("miffy".to_string());
    let mirror = Mirror::new(publisher);
    let dispatcher = Dispatcher::new(
        "http://127.0.0.1:3001".into(),
        "http://127.0.0.1:3000".into(),
        &["/api/{value}"],
    );

    let proxy = proxy::Service::new(dispatcher, mirror);

    proxy::run(proxy).await
}
