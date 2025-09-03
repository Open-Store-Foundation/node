use rdkafka::consumer::{Consumer, DefaultConsumerContext, MessageStream, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::{ClientConfig};
use tracing::info;

pub struct KfProducer {
    producer: FutureProducer,
}

impl KfProducer {

    pub fn new_client(
        brokers: String,
        client_id: Option<String>,
    ) -> KafkaResult<KfProducer> {
        info!("Configuring Kafka producer for brokers: {}", brokers);

        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", brokers);

        if let Some(id) = client_id {
            info!("Using Kafka client.id: {}", id);
            config.set("client.id", id);
        }

        let timeout = "5000";
        info!("Using Kafka message.timeout.ms: {}", timeout);
        config.set("message.timeout.ms", timeout);

        // Create the producer instance
        let producer: FutureProducer = config.create()?;
        info!("Kafka producer initialized successfully.");

        Ok(KfProducer { producer })
    }

    pub async fn send(&self, item: FutureRecord<'_, String, Vec<u8>>) -> OwnedDeliveryResult {
        self.producer.send(item, Timeout::Never)
            .await // TODO v2 timeout when add statistic
    }
}


pub struct KfConsumer {
    consumer: StreamConsumer,
}

pub type KfSubscription<'a> = MessageStream<'a, DefaultConsumerContext>;

impl KfConsumer {

    pub fn new_client(
        brokers: String,
        group_id: String,
    ) -> Option<KfConsumer> {
        // --- Конфигурация Consumer ---
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("auto.offset.reset", "earliest")
            .create()
            .ok()?;

        Some( KfConsumer { consumer })
    }

    pub fn subscribe(&self, topics: &[&str]) -> KafkaResult<KfSubscription> {
        self.consumer.subscribe(topics)?;

        Ok(
            self.consumer.stream()
        )
    }
}
