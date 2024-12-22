use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Kafka {
    /// brokers to connect to
    pub brokers: Vec<String>,

    /// topic where to publish requests
    pub topic: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub kafka: Kafka,

    /// default reference URL to use
    pub reference: String,
    /// default candidate URL to use
    pub candidate: String,

    /// port to listen to
    pub port: u16,

    /// whether to log in structured JSON format (otherwise: pretty human-readable
    pub log_json: bool,
}

impl Settings {
    pub(crate) fn emerge() -> Result<Settings, ConfigError> {
        let settings = Config::builder()
            .set_default("log_json", false)?
            .add_source(config::File::with_name("config.default.toml"))
            .add_source(config::File::with_name("config.toml"))
            .add_source(Environment::with_prefix("MIFFY"))
            .build();

        settings?.try_deserialize::<Settings>()
    }
}
