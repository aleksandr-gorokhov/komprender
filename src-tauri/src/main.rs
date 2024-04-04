// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::kafka_connection::KafkaConnection;

mod kafka_connection;
mod topic_commands;

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

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            connect,
            topic_commands::fetch_topics,
            topic_commands::fetch_topic,
            topic_commands::drop_topics,
            topic_commands::create_topic,
            disconnect,
            fetch_saved_brokers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
