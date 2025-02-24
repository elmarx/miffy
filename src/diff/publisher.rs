use crate::domain;
use crate::settings::Kafka;
use rdkafka::ClientConfig;
use rdkafka::producer::FutureRecord;
use std::time::Duration;
use tracing::{debug, info};

#[derive(Clone)]
pub struct Publisher {
    topic: String,
    producer: rdkafka::producer::FutureProducer,
}

impl Publisher {
    pub fn new(config: Kafka, properties: Vec<(String, String)>) -> Self {
        let mut cfg = ClientConfig::new();
        cfg.extend(config.properties.into_iter().map(|(k, v)| (k, v.into())));
        cfg.extend(properties);

        let producer = cfg.create().expect("invalid kafka configuration");

        Self {
            topic: config.topic,
            producer,
        }
    }

    pub async fn publish(&self, key: &str, sample: domain::Sample) {
        if sample.is_equal() {
            info!(
                "request to {} {} equals reference from {} to, not sending message",
                sample.request.method, sample.candidate.url, sample.reference.url
            );
            return;
        }

        let message = serde_json::to_string(&sample).expect("failed to serialize sample-message");

        let delivery_status = self
            .producer
            .send::<_, _, _>(
                FutureRecord::to(&self.topic).key(key).payload(&message),
                Duration::from_secs(0),
            )
            .await;
        debug!("Delivery status: {delivery_status:?}");
    }
}
