use crate::kafka_connection::{fetch_offsets, fulfill_tpl, KafkaConnection};
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rdkafka::consumer::BaseConsumer;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::BorrowedMessage,
    Message, Offset, TopicPartitionList,
};
use schema_registry_converter::async_impl::avro::AvroDecoder;
use serde::Serialize;
use serde_json::{self, Value as JsonValue};
use tauri::{AppHandle, Manager};
use tokio::sync::{mpsc, oneshot, oneshot::Sender, Mutex};
use tokio::time::Duration;

use crate::schema_registry::SchemaRegistry;

#[derive(Serialize)]
struct MessageResponse {
    key: String,
    value: JsonValue,
    partition: i32,
    offset: i64,
}

lazy_static! {
    static ref TX: Mutex<Vec<Sender<bool>>> = Mutex::new(vec![]);
}

#[tauri::command]
pub async fn stop_consumers() {
    println!("Stopping consumers");
    let mut signal_storage = TX.lock().await;
    let signals = signal_storage.drain(..).collect::<Vec<_>>();
    for tx in signals {
        let _ = tx.send(true);
    }
    signal_storage.clear();
}

#[tauri::command]
pub async fn consume_messages(
    app_handle: AppHandle,
    topic: String,
    mode: String,
) -> Result<(), String> {
    let avro_decoder = create_avro_decoder().await;

    let (tx, rx) = oneshot::channel::<bool>();
    {
        let mut signal_storage = TX.lock().await;
        signal_storage.push(tx);
    }
    let (tx_signal, mut rx_signal) = mpsc::channel::<()>(1);

    let client_config = KafkaConnection::get_client_config()
        .await
        .map_err(|e| e.to_string())?;
    let base_consumer: BaseConsumer = client_config.create().map_err(|e| {
        println!("Error creating consumer: {}", e);
        e.to_string()
    })?;

    let join_handle = tokio::spawn(async move {
        tokio::spawn(async move {
            if rx.await.is_ok() {
                let _ = tx_signal.send(()).await;
            }
        });

        let mut messages_count = 0;
        let group_id = generate_group_id();
        let consumer = create_consumer(group_id).await?;

        match consumer.subscribe(&[&*topic]) {
            Ok(result) => result,
            Err(err) => {
                let err = format!("Error subscribing to topic: {}", err);
                println!("Error subscribing to topic: {}", err);
                return Err(err);
            }
        };

        println!("Subscribed to topic: {}", topic);
        let mut seeked = false;
        while let Ok(message) = tokio::select! {
            message = consumer.recv() => message,
            _ = rx_signal.recv() => {
                println!("Cancellation signal received. Exiting...");
                return Ok::<(), String>(());
            },
        } {
            if !seeked {
                println!("Starting seek");
                seek(&consumer, &mode, &topic, &base_consumer).await?;
                println!("Seeked");
                seeked = true;
                continue;
            }
            process_message(&message, &avro_decoder, &app_handle).await?;
            messages_count += 1;

            if mode == "beginning" && messages_count >= 100 {
                break;
            }
        }
        Ok::<(), String>(())
    });

    let _ = join_handle.await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn seek(
    consumer: &StreamConsumer,
    mode: &str,
    topic: &str,
    base_consumer: &BaseConsumer,
) -> Result<(), String> {
    let assignment = consumer.assignment().map_err(|e| e.to_string())?;

    let offsets = match mode {
        "end" => {
            println!("Seeked to end of topic: {}", topic);
            let partitions = assignment.elements();
            for mut partition in partitions {
                partition
                    .set_offset(Offset::End)
                    .map_err(|e| e.to_string())?;
            }

            assignment
        }
        "beginning" => {
            println!("Starting to seek to beginning of topic: {}", topic);
            let partitions = assignment.elements();
            for mut partition in partitions {
                partition
                    .set_offset(Offset::Beginning)
                    .map_err(|e| e.to_string())?;
            }
            println!("Seeked to beginning of topic: {}", topic);
            assignment
        }
        "last" => {
            println!("Seeked to last 100 messages of topic: {}", topic);
            let partitions = assignment.elements();
            let partitions_count = partitions.len();
            println!("Partitions count: {:?}", partitions_count);

            let mut start_assignment = TopicPartitionList::new();
            let mut end_assignment = TopicPartitionList::new();

            for partition in partitions {
                fulfill_tpl(
                    &mut start_assignment,
                    topic,
                    partition.partition(),
                    Offset::Beginning,
                )?;
                fulfill_tpl(
                    &mut end_assignment,
                    topic,
                    partition.partition(),
                    Offset::End,
                )?;
            }
            println!("Partitions assigner, start fetching");
            let end_offsets = fetch_offsets(&base_consumer, end_assignment)?;
            println!("fetched end");
            let end_offsets = end_offsets.elements_for_topic(&topic);
            let start_offsets = fetch_offsets(&base_consumer, start_assignment)?;
            println!("fetched start");
            let start_offsets = start_offsets.elements_for_topic(&topic);

            let mut result_tpl = TopicPartitionList::new();

            for (index, partition) in end_offsets.iter().enumerate() {
                let high = match partition.offset() {
                    Offset::Offset(offset) => offset,
                    _ => 0,
                };
                let low = match start_offsets[index].offset() {
                    Offset::Offset(offset) => offset,
                    _ => 0,
                };
                let result = (high as f64 - f64::from(100) / partitions_count as f64).ceil() as i64;
                let final_result = std::cmp::max(result, low);
                println!("Seeked to offset: {}", final_result);
                fulfill_tpl(
                    &mut result_tpl,
                    topic,
                    partition.partition(),
                    Offset::Offset(final_result),
                )?;
            }

            result_tpl
        }
        _ => {
            let partitions = assignment.elements();
            let mut offsets = vec![];
            for partition in partitions {
                offsets.push((partition.partition(), Offset::Beginning));
            }

            assignment
        }
    };

    consumer
        .seek_partitions(offsets, Duration::from_secs(5))
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn generate_group_id() -> String {
    let rng = thread_rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    format!("__komprender-technical-consumer-group-{}", random_string)
}

async fn create_consumer(group_id: String) -> Result<StreamConsumer, String> {
    let mut client_config = KafkaConnection::get_client_config()
        .await
        .map_err(|e| e.to_string())?;

    client_config
        .set("group.id", group_id)
        .set("auto.offset.reset", "earliest")
        .create()
        .map_err(|e| {
            println!("Error creating consumer: {}", e);
            e.to_string()
        })
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

async fn process_message<'a>(
    message: &'a BorrowedMessage<'_>,
    avro_decoder: &Result<AvroDecoder<'_>, String>,
    app_handle: &AppHandle,
) -> Result<(), String> {
    match avro_decoder {
        Ok(decoder) => {
            if let Some(json) = decode_avro_to_json(message, &decoder).await {
                emit_message(app_handle, &json);
            } else if let Some(json) = decode_bytes_to_json(message)? {
                emit_message(app_handle, &json);
            }
        }
        Err(_) => {
            if let Some(json) = decode_bytes_to_json(message)? {
                emit_message(app_handle, &json);
            }
        }
    }
    Ok(())
}

fn emit_message(app_handle: &AppHandle, json: &MessageResponse) {
    app_handle
        .emit_all("message_received", json)
        .map_err(|e| eprintln!("Error emitting message event: {:?}", e))
        .ok();
}

fn get_message_data(message: &BorrowedMessage) -> (String, i32, i64) {
    let key = message
        .key()
        .map(|k| std::str::from_utf8(k).unwrap_or_default())
        .unwrap_or_default()
        .to_string();

    let partition = message.partition();
    let offset = message.offset();

    (key, partition, offset)
}

async fn decode_avro_to_json<'a>(
    message: &'a BorrowedMessage<'_>,
    avro_decoder: &'a AvroDecoder<'_>,
) -> Option<MessageResponse> {
    let (key, partition, offset) = get_message_data(message);

    match avro_decoder.decode(message.payload()).await {
        Ok(record) => {
            let json_value = JsonValue::try_from(record.value);
            match json_value {
                Ok(json) => Some(MessageResponse {
                    key,
                    value: json,
                    partition,
                    offset,
                }),
                Err(e) => {
                    eprintln!("Error converting Avro Value to JSON: {:?}", e);
                    None
                }
            }
        }
        _ => None,
    }
}

fn decode_bytes_to_json(message: &BorrowedMessage) -> Result<Option<MessageResponse>, String> {
    let (key, partition, offset) = get_message_data(message);

    let bytes = match message.payload() {
        Some(b) => b,
        None => return Err("Error getting message payload".to_string()),
    };
    let message_str = match std::str::from_utf8(bytes) {
        Ok(str) => str,
        Err(_) => return Err("Error decoding JSON message".to_string()),
    };

    match serde_json::from_str::<JsonValue>(message_str) {
        Ok(json) => Ok(Some(MessageResponse {
            key,
            value: json,
            partition,
            offset,
        })),
        Err(e) => {
            println!("Error parsing JSON: {}", e);
            Ok(Some(MessageResponse {
                key,
                value: message_str.into(),
                partition,
                offset,
            }))
        }
    }
}
