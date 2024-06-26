use crate::kafka_connection::{fetch_offsets, fulfill_tpl, KafkaConnection};
use rdkafka::admin::{AdminOptions, NewTopic, TopicReplication};
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::{Offset, TopicPartitionList};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Serialize, Deserialize)]
pub struct TopicResult {
    name: String,
    partitions: usize,
    messages: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Partition {
    id: i32,
    leader: i32,
    replicas: Vec<i32>,
    low: i64,
    high: i64,
    messages: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TopicPageResult {
    name: String,
    partitions: Vec<Partition>,
}

#[tauri::command]
pub async fn fetch_topics(filter: &str) -> Result<Vec<TopicResult>, String> {
    let kafka = KafkaConnection::get_consumer_instance().lock().await;

    let consumer = match &*kafka {
        Some(consumer_instance) => consumer_instance,
        None => return Err("Kafka connection not established".to_string()),
    };

    let metadata = match consumer.fetch_metadata(None, Duration::from_secs(10)) {
        Ok(metadata) => metadata,
        Err(e) => return Err(e.to_string()),
    };

    let mut topics_info = vec![];
    let mut end_assignment = TopicPartitionList::new();
    let mut start_assignment = TopicPartitionList::new();

    for topic in metadata.topics().iter() {
        let topic_name = topic.name();

        if !topic_name.contains(filter) {
            continue;
        }

        if topic_name.starts_with("_") {
            continue;
        }

        let partitions = topic.partitions().iter().len();
        let partition_ids = topic
            .partitions()
            .iter()
            .map(|p| p.id())
            .collect::<Vec<_>>();

        for partition in partition_ids.iter() {
            fulfill_tpl(
                &mut start_assignment,
                topic_name,
                *partition,
                Offset::Beginning,
            )?;
            fulfill_tpl(&mut end_assignment, topic_name, *partition, Offset::End)?;
        }

        let total_messages = 0;
        topics_info.push(TopicResult {
            name: topic_name.to_owned(),
            partitions,
            messages: total_messages,
        })
    }

    if topics_info.len() == 0 {
        return Ok(topics_info);
    }

    let inner_consumer = create_consumer().await?;

    let start_offsets = fetch_offsets(&inner_consumer, start_assignment)?;
    let end_offsets = fetch_offsets(&inner_consumer, end_assignment)?;

    for topic_info in topics_info.iter_mut() {
        let topic_name = &topic_info.name;
        let partition_start_offsets = start_offsets.elements_for_topic(topic_name);
        let partition_end_offsets = end_offsets.elements_for_topic(topic_name);
        for offset in partition_end_offsets.iter() {
            match offset.offset() {
                Offset::Offset(offset_num) => {
                    topic_info.messages += offset_num;
                }
                _ => {}
            }
        }
        for offset in partition_start_offsets.iter() {
            match offset.offset() {
                Offset::Offset(offset_num) => {
                    topic_info.messages -= offset_num;
                }
                _ => {}
            }
        }
    }

    Ok(topics_info)
}

async fn create_consumer() -> Result<BaseConsumer, String> {
    let client_config = KafkaConnection::get_client_config()
        .await
        .map_err(|e| e.to_string())?;

    client_config.create().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_topic(name: &str) -> Result<TopicPageResult, String> {
    let kafka = KafkaConnection::get_consumer_instance().lock().await;
    if let Some(consumer) = &*kafka {
        let metadata = consumer
            .fetch_metadata(Some(name), Duration::from_secs(10))
            .unwrap();

        let mut topic_info = TopicPageResult {
            name: name.to_owned(),
            partitions: vec![],
        };

        for topic in metadata.topics().iter() {
            let topic_name = topic.name();

            let mut end_assignment = TopicPartitionList::new();
            let mut start_assignment = TopicPartitionList::new();
            for partition_metadata in topic.partitions().iter() {
                fulfill_tpl(
                    &mut start_assignment,
                    topic_name,
                    partition_metadata.id(),
                    Offset::Beginning,
                )?;
                fulfill_tpl(
                    &mut end_assignment,
                    topic_name,
                    partition_metadata.id(),
                    Offset::End,
                )?;
                let partition = Partition {
                    id: partition_metadata.id(),
                    leader: partition_metadata.leader(),
                    replicas: partition_metadata.replicas().to_vec(),
                    low: 0,
                    high: 0,
                    messages: 0,
                };

                topic_info.partitions.push(partition);
            }
            let start_offsets = fetch_offsets(&consumer, start_assignment)?;
            let end_offsets = fetch_offsets(&consumer, end_assignment)?;

            for partition in topic_info.partitions.iter_mut() {
                let partition_start_offsets = start_offsets.elements_for_topic(&topic_name);
                let partition_end_offsets = end_offsets.elements_for_topic(&topic_name);

                for partition_elem in partition_end_offsets.iter() {
                    if partition_elem.partition() == partition.id {
                        match partition_elem.offset() {
                            Offset::Offset(offset_num) => {
                                partition.messages += offset_num;
                                partition.high = offset_num;
                            }
                            _ => {}
                        }
                    }
                }

                for partition_elem in partition_start_offsets.iter() {
                    if partition_elem.partition() == partition.id {
                        match partition_elem.offset() {
                            Offset::Offset(offset_num) => {
                                partition.messages -= offset_num;
                                partition.low = offset_num;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(topic_info)
    } else {
        Err("Kafka connection not established".to_string())
    }
}

#[tauri::command]
pub async fn drop_topics(topic_names: Vec<&str>) -> Result<(), String> {
    let admin_client = KafkaConnection::get_admin_client_instance().lock().await;

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
    let admin_client = KafkaConnection::get_admin_client_instance().lock().await;

    if let Some(admin_client) = &*admin_client {
        let opts = AdminOptions::new();

        let retention_time_str = topic.retention_time.to_string();
        let insync_replicas = topic.insync_replicas.to_string();
        let max_message_bytes = if topic.size_limit > 0 {
            topic.size_limit.to_string()
        } else {
            "1048588".to_string()
        };
        let cleanup_policy = topic.cleanup_policy.to_ascii_lowercase();
        let new_topic = NewTopic::new(
            topic.name,
            topic.partitions,
            TopicReplication::Fixed(topic.replication_factor),
        )
        .set("retention.ms", &retention_time_str)
        .set("cleanup.policy", &cleanup_policy)
        .set("min.insync.replicas", &insync_replicas)
        .set("max.message.bytes", &max_message_bytes);

        let result = admin_client
            .create_topics(&[new_topic], &opts)
            .await
            .map_err(|err| err.to_string())?;

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
