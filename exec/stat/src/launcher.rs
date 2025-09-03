use crate::data::models::ObjectEvent;
use crate::data::stat_buffer::StatBuffer;
use crate::data::stat_repo::StatRepo;
use crate::result::{StatError, StatResult};
use codegen_stat::stat::EventWrapper;
use prost::Message as ProstMessage;
use rdkafka::{message::BorrowedMessage, Message as RdkafkaMessage};
use std::{sync::Arc, time::Duration};
use futures::StreamExt;
use tracing::{debug, error, info, instrument, warn};

const BATCH_SIZE: usize = 1;

#[instrument(skip_all)]
pub fn launch_consumer(buffer: Arc<StatBuffer>, repo: Arc<StatRepo>) {
    tokio::task::spawn(async move {
        let mut message_stream = match buffer.events() {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to create Kafka message stream: {}", e);
                return;
            }
        };

        let mut batch: Vec<ObjectEvent> = Vec::with_capacity(BATCH_SIZE);

        info!("Starting Kafka consumer loop for stat events...");

        loop {
            match message_stream.next().await {
                Some(Ok(borrowed_message)) => {
                    match process_message(&borrowed_message) {
                        Ok(Some(event)) => {
                            debug!("New event received!");

                            batch.push(event);
                            if batch.len() >= BATCH_SIZE {
                                if let Err(e) = repo.insert_events(batch.clone()).await {
                                    error!("Failed to insert batch into ClickHouse: {}", e);
                                } else {
                                    info!(count = batch.len(), "Successfully inserted batch into ClickHouse");
                                    batch.clear();
                                }
                            }
                        }
                        Ok(None) => {
                        }
                        Err(e) => {
                            error!(
                                topic = borrowed_message.topic(),
                                partition = borrowed_message.partition(),
                                offset = borrowed_message.offset(),
                                "Failed to process message: {}. Skipping.",
                                e
                            );
                        }
                    }
                }
                Some(Err(kafka_error)) => {
                    error!("Kafka consumer error: {}", kafka_error);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                None => {
                    info!("Kafka message stream closed.");
                    if !batch.is_empty() {
                         if let Err(e) = repo.insert_events(batch.clone()).await {
                             error!("Failed to insert final batch into ClickHouse: {}", e);
                         } else {
                             info!(count = batch.len(), "Successfully inserted final batch into ClickHouse");
                         }
                    }
                    break;
                }
            }
        }
        info!("Kafka consumer loop finished.");
    });
}

#[instrument(skip(msg), fields(topic=msg.topic(), partition=msg.partition(), offset=msg.offset()))]
fn process_message(msg: &BorrowedMessage<'_>) -> StatResult<Option<ObjectEvent>> {
    match msg.payload() {
        Some(data) => {
            // let event_wrapper = EventWrapper::decode(data)?;

            info!("Decoded EventWrapper");
            match serde_json::from_slice::<ObjectEvent>(data) {
                 Ok(event) => {
                    // info!(event_name = event.event_name, object_id = event.object_id, "Converted to ObjectEvent");
                    Ok(Some(event))
                 },
                 Err(_) => {
                    warn!("Failed to convert EventWrapper to ObjectEvent (likely invalid enum values).");
                    Ok(None)
                 }
            }
        }
        None => {
            warn!("Received message with empty payload.");
            Ok(None)
        }
    }
}