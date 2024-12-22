use diff::dispatcher::Dispatcher;
use diff::mirror::Mirror;
use diff::publisher::Publisher;
use settings::Settings;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tracing::info;
use util::log;

mod diff;
mod domain;
mod http;
mod proxy;
mod settings;
mod util;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::emerge()?;

    log::init(settings.log_json);

    info!("{settings:?}");

    let publisher = Publisher::new(settings.kafka);
    let mirror = Mirror::new(publisher);
    let dispatcher = Dispatcher::new(
        settings.candidate.to_string(),
        settings.reference.to_string(),
        &["/api/{value}"],
    );

    let proxy = proxy::Service::new(dispatcher, mirror);

    proxy::run(settings.port, proxy).await?;

    Ok(())
}
