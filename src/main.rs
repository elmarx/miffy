use crate::settings::Setting;
use anyhow::Context;
use diff::dispatcher::Dispatcher;
use diff::mirror::Mirror;
use diff::publisher::Publisher;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tracing::info;
use util::log;

mod diff;
mod domain;
mod http;
mod jaq;
mod management;
mod model;
mod proxy;
mod settings;
mod util;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Setting::emerge().context("reading config")?;

    log::init(&settings.config.logging).await;

    info!("{settings:?}");

    let publisher = Publisher::new(settings.config.kafka, settings.kafka_properties);
    let mirror = Mirror::new(publisher);
    let dispatcher = Dispatcher::new(
        settings.config.reference.to_string(),
        settings.config.candidate.to_string(),
        settings.config.routes.as_slice(),
    )
    .context("creating dispatcher")?;

    let proxy = proxy::Service::new(dispatcher, mirror);

    tokio::task::spawn(proxy::run(settings.config.port, proxy));

    management::run(settings.config.management_port).await?;

    Ok(())
}
