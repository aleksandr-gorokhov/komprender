use std::path::PathBuf;

use once_cell::sync::Lazy;
use rdkafka::admin::AdminClient;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::FromClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer, StreamConsumer};
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

pub struct KafkaConnection;

#[derive(Serialize, Deserialize)]
struct Connections {
    items: Vec<ConnectionItem>,
}
#[derive(Serialize, Deserialize)]
pub struct ConnectionItem {
    kafka_broker: String,
    schema_registry: String,
    name: String,
}

pub static KAFKA_CONSUMER: Lazy<Mutex<Option<BaseConsumer>>> = Lazy::new(|| Mutex::new(None));
pub static KAFKA_STREAM_CONSUMER: Lazy<Mutex<Option<StreamConsumer>>> =
    Lazy::new(|| Mutex::new(None));
pub static KAFKA_PRODUCER: Lazy<Mutex<Option<FutureProducer>>> = Lazy::new(|| Mutex::new(None));
pub static KAFKA_ADMIN_CLIENT: Lazy<Mutex<Option<AdminClient<DefaultClientContext>>>> =
    Lazy::new(|| Mutex::new(None));
pub static BROKER: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

impl KafkaConnection {
    pub fn get_consumer_instance() -> &'static Mutex<Option<BaseConsumer>> {
        &KAFKA_CONSUMER
    }

    pub fn get_stream_consumer_instance() -> &'static Mutex<Option<StreamConsumer>> {
        &KAFKA_STREAM_CONSUMER
    }

    pub fn get_producer_instance() -> &'static Mutex<Option<FutureProducer>> {
        &KAFKA_PRODUCER
    }

    pub fn get_admin_client_instance() -> &'static Mutex<Option<AdminClient<DefaultClientContext>>>
    {
        &KAFKA_ADMIN_CLIENT
    }

    fn get_broker() -> &'static Mutex<Option<String>> {
        &BROKER
    }

    pub async fn get_client_config<'a>() -> Result<ClientConfig, String> {
        let mut config = ClientConfig::new();
        let broker = KafkaConnection::get_broker().lock().await;

        let broker = match &*broker {
            Some(broker) => broker.clone(),
            None => {
                return Err("Broker not set".to_string());
            }
        };

        config.set("bootstrap.servers", broker);

        Ok(config)
    }

    pub async fn disconnect() -> Result<(), String> {
        let mut kafka_consumer_guard = KafkaConnection::get_consumer_instance().lock().await;
        *kafka_consumer_guard = None;

        let mut kafka_admin_client_guard =
            KafkaConnection::get_admin_client_instance().lock().await;
        *kafka_admin_client_guard = None;

        let mut kafka_producer_guard = KafkaConnection::get_producer_instance().lock().await;
        *kafka_producer_guard = None;

        let mut kafka_stream_consumer_guard =
            KafkaConnection::get_stream_consumer_instance().lock().await;
        *kafka_stream_consumer_guard = None;

        let mut kafka_broker = KafkaConnection::get_broker().lock().await;
        *kafka_broker = None;

        Ok(())
    }

    pub async fn connect(host: &str, name: &str, schema_registry: &str) -> Result<bool, String> {
        if host.is_empty() {
            return Err("Broker cannot be empty".to_string());
        }

        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        let broker = if !host.contains(':') {
            format!("{}:9092", host)
        } else {
            host.to_string()
        };
        println!("Connecting to Kafka broker: {}", broker);
        let mut config = ClientConfig::new();
        let client_config = config.set("bootstrap.servers", broker.clone());
        let consumer: BaseConsumer = client_config
            .set("group.id", "technical")
            .create()
            .map_err(|e| e.to_string())?;
        let admin_client =
            AdminClient::from_config(&client_config).map_err(|err| err.to_string())?;
        let producer: FutureProducer = client_config.create().map_err(|e| e.to_string())?;
        let stream_consumer: StreamConsumer = client_config.create().map_err(|e| e.to_string())?;

        match consumer.fetch_metadata(None, std::time::Duration::from_secs(5)) {
            Ok(_) => println!(
                "Successfully connected and fetched metadata from Kafka broker: {}",
                broker
            ),
            Err(e) => {
                return Err(format!(
                    "Could not establish connection with kafka broker: {}. {}",
                    broker, e
                ))
            }
        }
        println!("Connected to Kafka broker: {}", broker);

        KafkaConnection::store_broker(&broker, name, schema_registry)
            .await
            .map_err(|e| e.to_string())?;

        let mut kafka_consumer_guard = KafkaConnection::get_consumer_instance().lock().await;
        match &*kafka_consumer_guard {
            Some(_) => {}
            None => {
                *kafka_consumer_guard = Some(consumer);
            }
        }

        let mut kafka_admin_client_guard =
            KafkaConnection::get_admin_client_instance().lock().await;

        match &*kafka_admin_client_guard {
            Some(_) => {}
            None => {
                *kafka_admin_client_guard = Some(admin_client);
            }
        }

        let mut kafka_producer_guard = KafkaConnection::get_producer_instance().lock().await;

        match &*kafka_producer_guard {
            Some(_) => {}
            None => {
                *kafka_producer_guard = Some(producer);
            }
        }

        let mut kafka_stream_consumer_guard =
            KafkaConnection::get_stream_consumer_instance().lock().await;

        match &*kafka_stream_consumer_guard {
            Some(_) => {}
            None => {
                *kafka_stream_consumer_guard = Some(stream_consumer);
            }
        }

        let mut broker_guard = KafkaConnection::get_broker().lock().await;
        match &*broker_guard {
            Some(_) => {}
            None => {
                *broker_guard = Some(broker.clone());
            }
        }

        Ok(true)
    }

    pub async fn get_saved_brokers() -> Result<Vec<ConnectionItem>, String> {
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

            let file_contents: Connections =
                serde_json::from_str(&contents).map_err(|e| e.to_string())?;

            Ok(file_contents.items)
        } else {
            Ok(vec![])
        }
    }

    async fn store_broker(host: &str, name: &str, schema_registry: &str) -> Result<(), String> {
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

            let mut file_contents: Connections =
                serde_json::from_str(&contents).map_err(|e| e.to_string())?;

            if !file_contents.items.iter().any(|item| {
                &item.kafka_broker.to_string() == &host.to_string()
                    && &item.schema_registry.to_string() == &schema_registry.to_string()
            }) {
                file_contents.items.push(ConnectionItem {
                    kafka_broker: host.to_string(),
                    schema_registry: schema_registry.to_string(),
                    name: name.to_string(),
                });
            }

            let serialized =
                serde_json::to_string_pretty(&file_contents).map_err(|e| e.to_string())?;
            fs::write(file_path, serialized)
                .await
                .map_err(|e| e.to_string())?;
        } else {
            let data = Connections {
                items: vec![ConnectionItem {
                    kafka_broker: host.to_string(),
                    schema_registry: schema_registry.to_string(),
                    name: name.to_string(),
                }],
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
