use rdkafka::admin::AdminOptions;
use rdkafka::consumer::Consumer;

use crate::kafka_connection::KafkaConnection;
use crate::TopicResult;

#[tauri::command]
pub async fn fetch_topics(filter: &str) -> Result<Vec<TopicResult>, String> {
    let kafka = KafkaConnection::get_consumer_instance().await.lock().await;
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

#[tauri::command]
pub async fn drop_topics(topic_names: Vec<&str>) -> Result<(), String> {
    let admin_client = KafkaConnection::get_admin_client_instance()
        .await
        .lock()
        .await;

    if let Some(admin_client) = &*admin_client {
        let opts = AdminOptions::new();

        admin_client
            .delete_topics(&topic_names, &opts)
            .await
            .map_err(|err| err.to_string())?;

        return Ok(());
    }
    Err("Kafka connection not established".to_string())
}
