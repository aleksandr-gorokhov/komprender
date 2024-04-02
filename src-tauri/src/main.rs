// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use rdkafka::consumer::Consumer;
use serde::{Deserialize, Serialize};

use crate::kafka_connection::KafkaConnection;

mod kafka_connection;

#[derive(Serialize, Deserialize)]
struct TopicResult {
    name: String,
    partitions: usize,
    messages: i64,
}

#[tauri::command]
async fn connect(broker: &str) -> Result<bool, String> {
    println!("Connecting to Kafka broker: {}", broker);
    KafkaConnection::connect(broker).await
}

#[tauri::command]
async fn disconnect() -> Result<(), String> {
    println!("Disconnecting from Kafka broker");

    KafkaConnection::disconnect().await
}

#[tauri::command]
async fn fetch_saved_brokers() -> Result<Vec<String>, String> {
    KafkaConnection::get_saved_brokers().await
}

#[tauri::command]
async fn fetch_topics(filter: &str) -> Result<Vec<TopicResult>, String> {
    let kafka = KafkaConnection::get_instance().await.lock().await;
    if let Some(consumer) = &*kafka {
        let metadata = consumer
            .fetch_metadata(None, std::time::Duration::from_secs(10))
            .unwrap();

        let mut topics_info = vec![];

        for topic in metadata.topics().iter() {
            let topic_name = topic.name();
            if !topic_name.contains(filter) {
                continue;
            }

            if topic_name.starts_with("_") {
                continue;
            }
            let mut total_messages = 0;
            for partition in topic.partitions().iter() {
                if let Ok((low, high)) = consumer.fetch_watermarks(
                    topic_name,
                    partition.id(),
                    std::time::Duration::from_secs(10),
                ) {
                    total_messages += high.saturating_sub(low);
                }
            }

            topics_info.push(TopicResult {
                name: topic_name.to_owned(),
                partitions: topic.partitions().iter().len(),
                messages: total_messages,
            });
        }
        Ok(topics_info)
    } else {
        Err("Kafka connection not established".to_string())
    }
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            connect,
            fetch_topics,
            disconnect,
            fetch_saved_brokers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
