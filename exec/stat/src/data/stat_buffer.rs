use std::sync::Arc;
use axum::http::StatusCode;
use rdkafka::error::{KafkaError, KafkaResult};
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use tracing::{error, info};
use db_kf::client::{KfConsumer, KfProducer, KfSubscription};

pub struct StatBuffer {
    topic: String,
    key: String,
    producer: Arc<KfProducer>,
    consumer: Arc<KfConsumer>,
}

impl StatBuffer {

    pub fn new(
        topic: String,
        key: String,
        kf_client: Arc<KfProducer>,
        kf_consumer: Arc<KfConsumer>
    ) -> Self {
        StatBuffer { topic, key, producer: kf_client, consumer: kf_consumer }
    }

    pub async fn publish(&self, data: Vec<u8>) -> KafkaResult<()> {
        let record = FutureRecord::to(self.topic.as_str())
            .payload(&data)
            .key(&self.key);

        info!(topic = %self.topic.as_str(), payload_size = data.len(), "Attempting to send message to Kafka");

        match self.producer.send(record).await {
            Ok(_) => {
                info!("Message successfully queued for Kafka topic {}", self.topic);
                Ok(())
            }

            Err((kafka_err, _owned_record)) => {
                error!(error = %kafka_err, topic = self.topic, "Failed to queue message for Kafka");
                Err(kafka_err)
            }
        }
    }

    pub fn events(&self) -> KafkaResult<KfSubscription> {
        self.consumer.subscribe(&[&self.topic])
    }
}
