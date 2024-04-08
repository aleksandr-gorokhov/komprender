// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::kafka_connection::KafkaConnection;
use crate::producer_commands::{produce_message_avro, produce_message_json};
use crate::schema_registry::{fetch_schema, fetch_sr_subjects, SchemaRegistry};
use crate::topic_commands::{create_topic, drop_topics, fetch_topic, fetch_topics};

mod kafka_connection;
mod producer_commands;
mod schema_registry;
mod topic_commands;

#[tauri::command]
async fn connect(broker: &str) -> Result<bool, String> {
    KafkaConnection::connect(broker).await?;
    SchemaRegistry::connect("").await
}

#[tauri::command]
async fn disconnect() -> Result<(), String> {
    println!("Disconnecting from Kafka and SR");

    KafkaConnection::disconnect().await?;
    SchemaRegistry::disconnect().await
}

#[tauri::command]
async fn fetch_saved_brokers() -> Result<Vec<String>, String> {
    KafkaConnection::get_saved_brokers().await
}

#[tokio::main]
async fn main() {
    let _ = fix_path_env::fix();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            connect,
            fetch_topics,
            fetch_topic,
            drop_topics,
            create_topic,
            produce_message_avro,
            produce_message_json,
            fetch_sr_subjects,
            fetch_schema,
            disconnect,
            fetch_saved_brokers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
