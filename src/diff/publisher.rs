use crate::domain;
use crate::settings::Kafka;
use rdkafka::producer::FutureRecord;
use rdkafka::ClientConfig;
use std::time::Duration;
use tracing::debug;

#[derive(Clone)]
pub struct Publisher {
    topic: String,
    producer: rdkafka::producer::FutureProducer,
}

impl Publisher {
    pub fn new(config: Kafka) -> Self {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", config.brokers.join(","))
            .set("message.timeout.ms", "5000")
            .create()
            .expect("invalid kafka configuration");

        Self {
            topic: config.topic,
            producer,
        }
    }

    pub async fn publish(&self, sample: domain::Sample) {
        let message = serde_json::to_string(&sample).expect("failed to serialize sample-message");

        let delivery_status = self
            .producer
            .send::<(), _, _>(
                FutureRecord::to(&self.topic).payload(&message),
                Duration::from_secs(0),
            )
            .await;
        debug!("Delivery status: {delivery_status:?}");
    }
}
