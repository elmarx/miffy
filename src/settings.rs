use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Kafka {
    /// brokers to connect to
    pub brokers: Vec<String>,

    /// topic where to publcish requests
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

    pub(crate) routes: Vec<Route>,
}

#[derive(Debug, Deserialize)]
pub struct Route {
    /// path in matchit-syntax (<https://docs.rs/matchit/latest/matchit/#parameters>). Names of parameters are irrelevant
    pub path: String,

    /// optional reference URL to use instead of the default-url
    pub reference: Option<String>,

    /// optional candidate URL to use instead of the default-url
    pub candidate: Option<String>,
}

impl Settings {
    pub(crate) fn emerge() -> Result<Settings, ConfigError> {
        let config_file = std::env::var("MIFFY_CONFIG").unwrap_or("config.toml".to_string());

        let settings = Config::builder()
            .set_default("port", 8080)?
            .set_default("log_json", false)?
            .set_default("routes", "[]")?
            .add_source(config::File::with_name(&config_file))
            .add_source(
                Environment::with_prefix("MIFFY")
                    .separator("_")
                    .list_separator(",")
                    .try_parsing(true)
                    .with_list_parse_key("kafka.brokers"),
            )
            .build();

        settings?.try_deserialize::<Settings>()
    }
}
