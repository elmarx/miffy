use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::collections::HashMap;

/// type to accept all values allowed by rdkafka.
/// rdkafka expects all properties as Into<String>, this enables to write numbers into toml without quotes
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum KafkaPropertyValue {
    String(String),
    Bool(bool),
    Integer(i64),
}

impl From<&KafkaPropertyValue> for String {
    fn from(v: &KafkaPropertyValue) -> Self {
        match v {
            KafkaPropertyValue::String(s) => s.clone(),
            KafkaPropertyValue::Bool(b) => b.to_string(),
            KafkaPropertyValue::Integer(i) => i.to_string(),
        }
    }
}

impl From<KafkaPropertyValue> for String {
    fn from(v: KafkaPropertyValue) -> Self {
        match v {
            KafkaPropertyValue::String(s) => s,
            KafkaPropertyValue::Bool(b) => b.to_string(),
            KafkaPropertyValue::Integer(i) => i.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Kafka {
    /// topic where to publish requests to
    pub topic: String,

    #[serde(flatten, default)]
    pub properties: HashMap<String, KafkaPropertyValue>,
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
            .add_source(Environment::with_prefix("MIFFY").separator("_"))
            .build();

        settings?.try_deserialize::<Settings>()
    }
}
