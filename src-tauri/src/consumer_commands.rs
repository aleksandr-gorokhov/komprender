use rdkafka::consumer::Consumer;
use rdkafka::Message;

use crate::kafka_connection::KafkaConnection;

enum EMode {
    From,
    Last,
    New,
}

#[tauri::command]
pub async fn consume_messages(
    topic: &str,
    from_beginning: bool,
    last_messages_count: usize,
    mode: &str,
) -> Result<(), String> {
    let topic = topic.to_owned();
    let join_handle = tokio::spawn(async move {
        let kafka = KafkaConnection::get_consumer_instance().lock().await;
        // let mut config = ClientConfig::new();
        // let client_config = config
        //     .set("bootstrap.servers", "kafka:9092")
        // let consumer: Result<BaseConsumer, String> =
        //     client_config.create().map_err(|e| e.to_string());
        let consumer = match &*kafka {
            Some(consumer) => consumer,
            None => {
                println!("Kafka connection not established");
                return;
            }
        };

        consumer
            .subscribe(&[&*topic])
            .expect("Failed to subscribe to topic");
        consumer.poll(std::time::Duration::from_secs(5));
        println!("Subscribed to topic: {}", topic);
        consumer
            .seek(
                &*topic,
                0,
                Offset::Beginning,
                std::time::Duration::from_secs(10),
            )
            .expect("Failed to start consumer");

        for message_result in consumer {
            let message = match message_result {
                Ok(message) => message,
                _ => continue,
            };

            let bytes = match message.payload() {
                Some(bytes) => bytes,
                None => continue,
            };

            let message_str = match std::str::from_utf8(bytes) {
                Ok(str) => str,
                Err(_) => {
                    println!("Error decoding message");
                    continue;
                }
            };

            println!("Message: {}", message_str);

            match serde_json::from_str::<serde_json::Value>(message_str) {
                Ok(json) => println!("Message: {}", json),
                Err(e) => println!("Error parsing JSON: {}", e),
            }
        }
    });

    join_handle.await.map_err(|e| e.to_string())?;
    Ok(())
}
