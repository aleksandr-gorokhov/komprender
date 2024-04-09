use crate::kafka_connection::KafkaConnection;
use apache_avro::types::Value;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::BorrowedMessage;
use rdkafka::Message;
use schema_registry_converter::async_impl::avro::AvroDecoder;
use serde_json::{self, Value as JsonValue};
use tauri::{AppHandle, Manager};

use crate::schema_registry::SchemaRegistry;

#[tauri::command]
pub async fn consume_messages(
    app_handle: AppHandle,
    topic: String,
    from_beginning: bool,
    _last_messages_count: usize,
    _mode: &str,
) -> Result<(), String> {
    let avro_decoder = create_avro_decoder().await?;

    let group_id = generate_group_id();

    let join_handle = tokio::spawn(async move {
        let consumer = create_consumer(group_id, from_beginning).await?;

        match consumer.subscribe(&[&*topic]) {
            Ok(result) => result,
            Err(err) => {
                let err = format!("Error subscribing to topic: {}", err);
                println!("Error subscribing to topic: {}", err);
                return Err(err);
            }
        };

        println!("Subscribed to topic: {}", topic);

        while let Ok(message) = consumer.recv().await {
            process_message(&message, &avro_decoder, &app_handle).await?
        }
        Ok::<(), String>(())
    });

    let _ = join_handle.await.map_err(|e| e.to_string())?;
    Ok(())
}

fn generate_group_id() -> String {
    let mut rng = thread_rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    format!("massive-consumer-{}", random_string)
}

async fn create_consumer(group_id: String, from_beginning: bool) -> Result<StreamConsumer, String> {
    let mut client_config = KafkaConnection::get_client_config()
        .await
        .map_err(|e| e.to_string())?;

    let offset = if from_beginning { "earliest" } else { "latest" };
    client_config
        .set("group.id", &group_id)
        .set("auto.offset.reset", offset)
        .create()
        .map_err(|e| e.to_string())
}

async fn create_avro_decoder<'a>() -> Result<AvroDecoder<'a>, String> {
    let sr_settings_guard = SchemaRegistry::get_settings().lock().await;
    match &*sr_settings_guard {
        Some(settings) => Ok(AvroDecoder::new(settings.clone())),
        None => {
            let err = "Error getting schema registry settings".to_string();
            return Err(err);
        }
    }
}

async fn get_avro_value<'a>(
    msg: &'a BorrowedMessage<'_>,
    decoder: &'a AvroDecoder<'_>,
) -> Result<Value, String> {
    match decoder.decode(msg.payload()).await {
        Ok(r) => Ok(r.value),
        Err(e) => Err(format!("Error getting value: {}", e)),
    }
}

async fn process_message<'a>(
    message: &'a BorrowedMessage<'_>,
    avro_decoder: &'a AvroDecoder<'_>,
    app_handle: &AppHandle,
) -> Result<(), String> {
    if let Some(json) = decode_avro_to_json(message, avro_decoder).await {
        emit_message(app_handle, &json);
    } else if let Some(json) = decode_bytes_to_json(message)? {
        emit_message(app_handle, &json);
    }
    Ok(())
}

fn emit_message(app_handle: &AppHandle, json: &(String, JsonValue)) {
    app_handle
        .emit_all("message_received", json)
        .map_err(|e| eprintln!("Error emitting message event: {:?}", e))
        .ok();
}

fn get_key(message: &BorrowedMessage) -> String {
    message
        .key()
        .map(|k| std::str::from_utf8(k).unwrap_or_default())
        .unwrap_or_default()
        .to_string()
}

async fn decode_avro_to_json<'a>(
    message: &'a BorrowedMessage<'_>,
    avro_decoder: &'a AvroDecoder<'_>,
) -> Option<(String, JsonValue)> {
    let key = get_key(message);

    match avro_decoder.decode(message.payload()).await {
        Ok(record) => {
            let json_value = JsonValue::try_from(record.value);
            match json_value {
                Ok(json) => Some((key, json)),
                Err(e) => {
                    eprintln!("Error converting Avro Value to JSON: {:?}", e);
                    None
                }
            }
        }
        _ => None,
    }
}

fn decode_bytes_to_json(message: &BorrowedMessage) -> Result<Option<(String, JsonValue)>, String> {
    let key = get_key(message);

    let bytes = match message.payload() {
        Some(b) => b,
        None => return Err("Error getting message payload".to_string()),
    };
    let message_str = match std::str::from_utf8(bytes) {
        Ok(str) => str,
        Err(_) => return Err("Error decoding JSON message".to_string()),
    };

    match serde_json::from_str::<JsonValue>(message_str) {
        Ok(json) => Ok(Some((key, json))),
        Err(e) => {
            println!("Error parsing JSON: {}", e);
            Ok(None)
        }
    }
}
