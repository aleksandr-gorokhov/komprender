use once_cell::sync::Lazy;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};

pub struct KafkaConsumerSingleton {
    consumer: BaseConsumer,
}

impl KafkaConsumerSingleton {
    pub fn get_instance() -> &'static BaseConsumer {
        static INSTANCE: Lazy<KafkaConsumerSingleton> = Lazy::new(|| {
            let consumer: BaseConsumer = ClientConfig::new()
                .set("bootstrap.servers", "kafka:9092")
                .create()
                .expect("Failed to create Kafka consumer");
            KafkaConsumerSingleton { consumer }
        });

        &INSTANCE.consumer
    }
}
