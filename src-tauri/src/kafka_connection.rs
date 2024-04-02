use once_cell::sync::Lazy;
use rdkafka::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use tokio::sync::Mutex;

pub struct KafkaConnection;

pub static KAFKA_CONNECTION: Lazy<Mutex<Option<BaseConsumer>>> = Lazy::new(|| Mutex::new(None));

impl KafkaConnection {
    pub async fn get_instance() -> &'static Mutex<Option<BaseConsumer>> {
        &KAFKA_CONNECTION
    }

    pub async fn disconnect() -> Result<(), String> {
        let mut kafka_consumer_guard = KafkaConnection::get_instance().await.lock().await;
        *kafka_consumer_guard = None;

        Ok(())
    }

    pub async fn connect(broker: &str) -> Result<bool, String> {
        println!("Connecting to Kafka broker: {}", broker);
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", broker)
            .create()
            .map_err(|e| e.to_string())?;

        match consumer.fetch_metadata(None, std::time::Duration::from_secs(1)) {
            Ok(_) => println!(
                "Successfully connected and fetched metadata from Kafka broker: {}",
                broker
            ),
            Err(_) => {
                return Err(format!(
                    "Could not establish connection with kafka broker: {}",
                    broker
                ))
            }
        }
        println!("Connected to Kafka broker: {}", broker);

        let mut kafka_consumer_guard = KafkaConnection::get_instance().await.lock().await;
        match &*kafka_consumer_guard {
            Some(_) => Ok(true),
            None => {
                *kafka_consumer_guard = Some(consumer);
                Ok(true)
            }
        }
    }
}
