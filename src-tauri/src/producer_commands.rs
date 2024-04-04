use apache_avro::types::Value;
use rdkafka::producer::FutureRecord;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy::RecordNameStrategy;
use tokio::time::Duration;

use crate::kafka_connection::KafkaConnection;

#[tauri::command]
pub async fn produce_message() -> Result<(), String> {
    let sr_settings = SrSettings::new_builder(String::from("http://localhost:8081"))
        .set_timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let encoder = AvroEncoder::new(sr_settings);
    let key_strategy = RecordNameStrategy("alerts-value".to_string());
    let bytes = encoder
        .encode(
            vec![
                ("id", Value::String("asd".to_string())),
                ("createdAt", Value::Long(1712260161977)),
                ("payload", Value::Bytes("Alice".as_bytes().to_vec())),
                ("type", Value::String("Ebasos".to_string())),
            ],
            key_strategy,
        )
        .await;

    if let Ok(bytes) = bytes {
        let producer = KafkaConnection::get_producer_instance().await.lock().await;
        if let Some(producer) = &*producer {
            let topic = "huge-topic";
            let produce_future = producer.send(
                FutureRecord::to(topic).key(&()).payload(&bytes),
                Duration::from_secs(10),
            );

            match produce_future.await {
                Ok(delivery) => println!("Sent: {:?}", delivery),
                Err((e, _)) => println!("Error: {:?}", e),
            }
        }
    } else if let Err(e) = bytes {
        return Err(e.to_string());
    }

    Ok(())
}
