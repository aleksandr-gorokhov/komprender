use rdkafka::admin::{AdminOptions, NewTopic, TopicReplication};
use rdkafka::consumer::Consumer;
use serde::{Deserialize, Serialize};

use crate::kafka_connection::KafkaConnection;
use crate::TopicResult;

#[derive(Debug, Serialize, Deserialize)]
pub enum CleanupPolicy {
    Delete,
    Compact,
    CompactDelete,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic<'a> {
    name: &'a str,
    partitions: i32,
    cleanup_policy: &'a str,
    insync_replicas: usize,
    replication_factor: i32,
    retention_time: usize,
    size_limit: usize,
}

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

#[tauri::command]
pub async fn create_topic<'a>(topic: Topic<'a>) -> Result<(), String> {
    println!("{:?}", topic);
    let admin_client = KafkaConnection::get_admin_client_instance()
        .await
        .lock()
        .await;

    if let Some(admin_client) = &*admin_client {
        let opts = AdminOptions::new();

        let retention_time_str = topic.retention_time.to_string();
        let cleanup_policy = topic.cleanup_policy.to_ascii_lowercase();
        let new_topic = NewTopic::new(
            topic.name,
            topic.partitions,
            TopicReplication::Fixed(topic.replication_factor),
        )
        .set("retention.ms", &retention_time_str)
        .set("cleanup.policy", &cleanup_policy);

        let result = admin_client
            .create_topics(&[new_topic], &opts)
            .await
            .map_err(|err| err.to_string())?;

        println!("{:?}", result);
        match result.len() {
            1 => match &result[0] {
                Ok(_) => {
                    println!("Topic {} created", topic.name);
                }
                Err(e) => {
                    return Err(e.1.to_string());
                }
            },
            _ => {
                return Err("Unknown error".to_string());
            }
        }

        return Ok(());
    }
    Err("Kafka connection not established".to_string())
}
