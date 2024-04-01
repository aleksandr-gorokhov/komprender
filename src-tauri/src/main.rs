// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rdkafka::{ClientConfig, TopicPartitionList};
use rdkafka::consumer::{BaseConsumer, Consumer};
use serde::{Deserialize, Serialize};

use kafka_connection::KafkaConsumerSingleton;

mod kafka_connection;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Deserialize)]
struct TopicResult {
    name: String,
    partitions: usize,
    messages: i64,
}

#[tauri::command]
fn fetch_topics(filter: &str) -> Vec<TopicResult> {
    let consumer = KafkaConsumerSingleton::get_instance();

    let metadata = consumer
        .fetch_metadata(None, std::time::Duration::from_secs(10))
        .unwrap();

    let mut topics_info = vec![];

    for topic in metadata.topics().iter() {
        if !topic.name().contains(filter) {
            continue;
        }
        let mut total_messages = 0;
        for partition in topic.partitions().iter() {
            if let Ok((low, high)) = consumer.fetch_watermarks(
                topic.name(),
                partition.id(),
                std::time::Duration::from_secs(10),
            ) {
                total_messages += high.saturating_sub(low);
            }
        }

        topics_info.push(TopicResult {
            name: topic.name().to_owned(),
            partitions: topic.partitions().iter().len(),
            messages: total_messages,
        });
    }

    topics_info
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![fetch_topics])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
