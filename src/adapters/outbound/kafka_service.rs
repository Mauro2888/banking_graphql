use rdkafka::producer::FutureProducer;
use rdkafka::consumer::StreamConsumer;
use std::sync::Arc;
use std::time::Duration;
use rdkafka::producer::FutureRecord;
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use async_graphql::futures_util::StreamExt;

pub struct KafkaService {
    producer: Arc<FutureProducer>,
    consumer: Arc<StreamConsumer>,
}

impl KafkaService {
    /// Crea il service con producer e consumer
    pub fn new(
        producer: Arc<FutureProducer>,
        consumer: Arc<StreamConsumer>,
    ) -> Self {
        KafkaService { producer, consumer }
    }

    /// Pubblica un evento
    pub async fn publish(
        &self,
        topic: &str,
        key: &str,
        payload: &serde_json::Value,
    ) -> Result<(i32, i64), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(payload)?;
        let record = FutureRecord::to(topic).key(key).payload(&json);

        match self.producer.send(record, Duration::from_secs(5)).await {
            Ok((partition, offset)) => {
                tracing::info!("✓ Published to {}:{}", partition, offset);
                Ok((partition, offset))
            }
            Err((e, _)) => {
                tracing::error!("❌ Publish failed: {}", e);
                Err(e.into())
            }
        }
    }

    /// Avvia il consumer loop in background
    pub fn start_consuming(self: Arc<Self>) {
        tokio::spawn(async move {
            if let Err(e) = self.consume_loop().await {
                tracing::error!("❌ Consumer crashed: {}", e);
            }
        });
    }

    /// Loop interno di consumo
    async fn consume_loop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.subscribe(&["user-events"])?;
        tracing::info!("📡 Kafka consumer started on 'user-events'");

        let mut stream = self.consumer.stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let Some(payload) = message.payload_view::<str>().and_then(|r| r.ok()) {
                        tracing::info!("📨 Received: {}", payload);

                        match self.process_event(payload).await {
                            Ok(_) => {
                                self.consumer.commit_message(&message, rdkafka::consumer::CommitMode::Async).ok();
                            }
                            Err(e) => tracing::error!("Processing failed: {}", e),
                        }
                    }
                }
                Err(e) => tracing::error!("Kafka error: {}", e),
            }
        }

        Ok(())
    }

    /// Elabora un evento (tu puoi metterci la logica che vuoi)
    async fn process_event(&self, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
        let event: serde_json::Value = serde_json::from_str(payload)?;
        tracing::info!("Event type: {:?}", event.get("event_type"));
        // Qui puoi fare cosa vuoi: repository calls, etc
        Ok(())
    }
}