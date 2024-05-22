// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::consumer_commands::{consume_messages, stop_consumers};
use crate::kafka_connection::{ConnectionItem, KafkaConnection};
use crate::producer_commands::{produce_message_avro, produce_message_json};
use crate::schema_registry::{fetch_schema, fetch_sr_subjects, SchemaRegistry};
use crate::topic_commands::{create_topic, drop_topics, fetch_topic, fetch_topics};
use tauri::api::dialog::confirm;
use tauri::{AppHandle, Manager};

mod consumer_commands;
mod kafka_connection;
mod producer_commands;
mod schema_registry;
mod topic_commands;

const CURRENT_VERSION: &str = "1.3.0";

#[tauri::command]
async fn connect(host: &str, name: &str, schema_registry: &str) -> Result<bool, String> {
    SchemaRegistry::connect(schema_registry).await?;
    KafkaConnection::connect(host, name, schema_registry).await
}

#[tauri::command]
async fn disconnect() -> Result<(), String> {
    println!("Disconnecting from Kafka and SR");

    KafkaConnection::disconnect().await?;
    SchemaRegistry::disconnect().await
}

#[tauri::command]
async fn fetch_saved_brokers() -> Result<Vec<ConnectionItem>, String> {
    KafkaConnection::get_saved_brokers().await
}

#[tauri::command]
async fn check_version(app_handle: AppHandle) -> Result<(), String> {
    let versions = reqwest::get("https://raw.githubusercontent.com/aleksandr-gorokhov/versions/main/komprender-versions.json")
        .await.map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await.map_err(|e| e.to_string())?;

    let version_status = match versions.get(CURRENT_VERSION) {
        Some(status) => status.clone(),
        None => {
            println!("Version {} not found in the JSON data.", CURRENT_VERSION);
            return Err("Version not found".to_string());
        }
    };

    let main_window = app_handle.get_window("main").unwrap();

    if version_status == "unsafe" {
        confirm(Some(&main_window),
                "Warning",
                "Your version is marked as unsafe. Application will shutdown. Consider updating the application",
                move |_| {
            app_handle.exit(0);
        });
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = fix_path_env::fix();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            connect,
            check_version,
            fetch_topics,
            fetch_topic,
            drop_topics,
            create_topic,
            produce_message_avro,
            produce_message_json,
            consume_messages,
            stop_consumers,
            fetch_sr_subjects,
            fetch_schema,
            disconnect,
            fetch_saved_brokers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
