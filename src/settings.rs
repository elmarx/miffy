use std::collections::HashMap;

use crate::util::log;
use config::{ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("../config.default.toml");

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
pub struct Config {
    pub kafka: Kafka,

    /// default reference URL to use
    pub reference: String,
    /// default candidate URL to use
    pub candidate: String,

    /// port to listen to
    pub port: u16,

    pub management_port: u16,

    /// format to log.
    pub logging: log::Format,

    pub routes: Vec<Route>,
}

#[derive(Debug)]
pub struct Setting {
    pub config: Config,
    pub kafka_properties: Vec<(String, String)>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    /// path in matchit-syntax (<https://docs.rs/matchit/latest/matchit/#parameters>). Names of parameters are irrelevant
    pub path: String,

    /// name of the param (from the route) to use as
    pub key: Option<String>,

    /// optional settings for reference
    pub reference: Option<Upstream>,
    /// optional settings for candidate
    pub candidate: Option<Upstream>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Upstream {
    /// base-URL to use instead of the default-url (the path will be appended)
    pub base: Option<String>,
    /// JQ-filter to transform the response before comparison and publishing  
    pub pre_transform: Option<String>,
    /// JQ-filter to transform the response after comparison (i.e.: the original response will be used for comparison)
    /// but before publishing.
    pub post_transform: Option<String>,
    /// JQ-filter to transform the response for comparison only. Use this to e.g. exclude debugging-information or sort the response
    /// this will also  
    pub canonicalize: Option<String>,
}

impl Setting {
    pub(crate) fn emerge() -> Result<Setting, ConfigError> {
        let config_file = std::env::var("MIFFY_CONFIG").unwrap_or("config.toml".to_string());

        let settings = config::Config::builder()
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_source(config::File::with_name(&config_file))
            .add_source(Environment::with_prefix("MIFFY").separator("_"))
            .build();

        let kafka_properties = kafka_from_env(std::env::vars());

        settings?.try_deserialize::<Config>().map(|config| Setting {
            config,
            kafka_properties,
        })
    }
}

#[cfg(test)]
impl TryFrom<&str> for Config {
    type Error = ConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        config::Config::builder()
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_source(File::from_str(value, FileFormat::Toml))
            .build()?
            .try_deserialize()
    }
}

/// collect env-vars into kafka-properties
/// e.g. turns `KAFKA_BOOTSTRAP_SERVERS` into `bootstrap.servers`
fn kafka_from_env(env_vars: impl Iterator<Item = (String, String)>) -> Vec<(String, String)> {
    env_vars
        .filter_map(|(k, v)| {
            k.strip_prefix("KAFKA_").map(|prop| {
                (
                    prop.replace('_', ".").to_lowercase().to_string(),
                    v.to_string(),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::{Config, Upstream, kafka_from_env};

    #[test]
    fn test_kafka_from_env() {
        let env_vars = vec![
            ("XYZ".to_string(), "short".to_string()),
            (
                "KAFKA_BOOTSTRAP_SERVERS".to_string(),
                "localhost:9092".to_string(),
            ),
            ("KAFKA_GROUP_ID".to_string(), "miffy".to_string()),
            (
                "KAFKA_SSL_CA_LOCATION".to_string(),
                "/var/run/secrets/ca.pem".to_string(),
            ),
        ];

        let actual = kafka_from_env(env_vars.into_iter());
        let expected: Vec<_> = vec![
            ("bootstrap.servers", "localhost:9092"),
            ("group.id", "miffy"),
            ("ssl.ca.location", "/var/run/secrets/ca.pem"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_full_upstream() {
        let config = r#"
        reference = "http://127.0.0.1:3000"
        candidate = "http://127.0.0.1:3001" 

        [[routes]]
        path = "/api/v1/echo"
        key = 'echo'
        reference = { pre_transform = ".", post_transform = ".",  canonicalize = "."}
        "#;

        let actual = Config::try_from(config).expect("should be ok");
        let route = &actual.routes[0];
        assert_eq!(
            route.reference,
            Some(Upstream {
                base: None,
                pre_transform: Some(".".to_string()),
                post_transform: Some(".".to_string()),
                canonicalize: Some(".".to_string()),
            })
        )
    }
}
