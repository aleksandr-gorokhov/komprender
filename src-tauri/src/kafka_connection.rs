use std::path::PathBuf;

use once_cell::sync::Lazy;
use rdkafka::admin::AdminClient;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::FromClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

pub struct KafkaConnection;

#[derive(Serialize, Deserialize)]
struct Brokers {
    brokers: Vec<String>,
}

pub static KAFKA_CONSUMER: Lazy<Mutex<Option<BaseConsumer>>> = Lazy::new(|| Mutex::new(None));
pub static KAFKA_PRODUCER: Lazy<Mutex<Option<FutureProducer>>> = Lazy::new(|| Mutex::new(None));
pub static KAFKA_ADMIN_CLIENT: Lazy<Mutex<Option<AdminClient<DefaultClientContext>>>> =
    Lazy::new(|| Mutex::new(None));

impl KafkaConnection {
    pub async fn get_consumer_instance() -> &'static Mutex<Option<BaseConsumer>> {
        &KAFKA_CONSUMER
    }

    pub async fn get_producer_instance() -> &'static Mutex<Option<FutureProducer>> {
        &KAFKA_PRODUCER
    }

    pub async fn get_admin_client_instance(
    ) -> &'static Mutex<Option<AdminClient<DefaultClientContext>>> {
        &KAFKA_ADMIN_CLIENT
    }

    pub async fn disconnect() -> Result<(), String> {
        let mut kafka_consumer_guard = KafkaConnection::get_consumer_instance().await.lock().await;
        *kafka_consumer_guard = None;

        let mut kafka_admin_client_guard = KafkaConnection::get_admin_client_instance()
            .await
            .lock()
            .await;
        *kafka_admin_client_guard = None;

        let mut kafka_producer_guard = KafkaConnection::get_admin_client_instance()
            .await
            .lock()
            .await;
        *kafka_producer_guard = None;

        Ok(())
    }

    pub async fn connect(broker: &str) -> Result<bool, String> {
        if broker.is_empty() {
            return Err("Broker cannot be empty".to_string());
        }

        if broker.contains(',') {
            return Err("Cluster connection are not supported yet".to_string());
        }

        let broker = if !broker.contains(':') {
            format!("{}:9092", broker)
        } else {
            broker.to_string()
        };

        println!("Connecting to Kafka broker: {}", broker);
        let mut config = ClientConfig::new();
        let client_config = config.set("bootstrap.servers", broker.clone());
        let consumer: BaseConsumer = client_config.create().map_err(|e| e.to_string())?;
        let admin_client =
            AdminClient::from_config(&client_config).map_err(|err| err.to_string())?;
        let producer: FutureProducer = client_config.create().map_err(|e| e.to_string())?;

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

        KafkaConnection::store_broker(&broker)
            .await
            .map_err(|e| e.to_string())?;

        let mut kafka_consumer_guard = KafkaConnection::get_consumer_instance().await.lock().await;
        match &*kafka_consumer_guard {
            Some(_) => {}
            None => {
                *kafka_consumer_guard = Some(consumer);
            }
        }

        let mut kafka_admin_client_guard = KafkaConnection::get_admin_client_instance()
            .await
            .lock()
            .await;

        match &*kafka_admin_client_guard {
            Some(_) => {}
            None => {
                *kafka_admin_client_guard = Some(admin_client);
            }
        }

        let mut kafka_producer_guard = KafkaConnection::get_producer_instance().await.lock().await;

        match &*kafka_producer_guard {
            Some(_) => {}
            None => {
                *kafka_producer_guard = Some(producer);
            }
        }

        Ok(true)
    }

    pub async fn get_saved_brokers() -> Result<Vec<String>, String> {
        let local_data_dir = tauri::api::path::local_data_dir()
            .unwrap_or(PathBuf::new())
            .display()
            .to_string();

        let file_path = PathBuf::from(local_data_dir).join("komprender/brokers.json");

        if file_path.exists() {
            let mut file = File::open(&file_path).await.map_err(|e| e.to_string())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .await
                .map_err(|e| e.to_string())?;

            let file_contents: Brokers =
                serde_json::from_str(&contents).map_err(|e| e.to_string())?;

            Ok(file_contents.brokers)
        } else {
            Ok(vec![])
        }
    }

    async fn store_broker(broker: &str) -> Result<(), String> {
        let local_data_dir = tauri::api::path::local_data_dir()
            .unwrap_or(PathBuf::new())
            .display()
            .to_string();

        let file_path = PathBuf::from(local_data_dir).join("komprender/brokers.json");

        if file_path.exists() {
            let mut file = File::open(&file_path).await.map_err(|e| e.to_string())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .await
                .map_err(|e| e.to_string())?;

            let mut file_contents: Brokers =
                serde_json::from_str(&contents).map_err(|e| e.to_string())?;

            if !file_contents.brokers.contains(&broker.to_string()) {
                file_contents.brokers.push(broker.to_string());
            }

            let serialized =
                serde_json::to_string_pretty(&file_contents).map_err(|e| e.to_string())?;
            fs::write(file_path, serialized)
                .await
                .map_err(|e| e.to_string())?;
        } else {
            let data = Brokers {
                brokers: vec![broker.to_string()],
            };

            let serialized = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            let mut file = File::create(&file_path).await.map_err(|e| e.to_string())?;
            file.write_all(serialized.as_bytes())
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
