// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rdkafka::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn fetch_topics() -> Vec<String> {
    let consumer: BaseConsumer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .create()
        .unwrap_or_else(|err| {
            eprintln!("Failed to create Kafka consumer: {}", err);
            std::process::exit(1);
        });

    let metadata = consumer.fetch_metadata(None, std::time::Duration::from_secs(10)).unwrap();

    metadata.topics().iter().map(|topic| topic.name().to_owned()).collect()
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![fetch_topics])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
