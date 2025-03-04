use serde::Deserialize;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// log in JSON Format
    Json,
    /// log in pretty, human-readable format
    Human,
}

/// initialize the tracing subscriber.
pub fn init(format: &Format) {
    match format {
        Format::Json => tracing_subscriber::fmt()
            .json()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .init(),
        Format::Human => tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .init(),
    }
}
