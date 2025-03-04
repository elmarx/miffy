use serde::Deserialize;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// log in JSON Format
    Json,
    /// log in pretty, human-readable format
    Human,

    /// google cloud compatible structured logging: <https://cloud.google.com/logging/docs/structured-logging>
    Stackdriver,
}

/// initialize the tracing subscriber.
pub fn init(format: &Format) {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    match format {
        Format::Json => tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init(),
        Format::Human => tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(env_filter)
            .init(),
        Format::Stackdriver => {
            let stackdriver = tracing_stackdriver::layer();
            tracing_subscriber::registry()
                .with(stackdriver)
                .with(env_filter)
                .init()
        }
    }
}
