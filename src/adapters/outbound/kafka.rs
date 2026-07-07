use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, StreamConsumer};
use rdkafka::producer::FutureProducer;

pub fn create_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("acks", "all")
        .create()
        .expect("Producer creation error")
}

pub fn create_consumer(brokers: &str, group_id: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Consumer creation failed")
}
