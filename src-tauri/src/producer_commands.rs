use apache_avro::types::Value;
use rdkafka::producer::FutureRecord;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy::RecordNameStrategy;
use serde_json::{Map as JsonMap, Value as JsonValue};
use tokio::time::Duration;

use crate::kafka_connection::KafkaConnection;

#[tauri::command]
pub async fn produce_message_avro(
    topic: &str,
    payload: &str,
    schema_name: &str,
    key: &str,
) -> Result<(), String> {
    println!("{}", schema_name);
    let parsed_json: JsonValue = serde_json::from_str(payload).map_err(|e| {
        println!("Error parsing JSON: {}", e);
        e.to_string()
    })?;
    let sr_settings = SrSettings::new_builder(String::from("http://localhost:8081"))
        .set_timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let key_strategy = RecordNameStrategy(schema_name.to_string());

    let encoder = AvroEncoder::new(sr_settings.clone());

    let bytes = match parsed_json {
        serde_json::Value::Object(map) => encoder.encode_struct(map, &key_strategy).await,
        _ => return Err("Payload must be a JSON object".to_string().into()),
    };

    if let Ok(bytes) = bytes {
        println!("success encoding");
        let producer = KafkaConnection::get_producer_instance().lock().await;
        if let Some(producer) = &*producer {
            let produce_future = producer.send(
                FutureRecord::to(topic).key(key).payload(&bytes),
                Duration::from_secs(10),
            );

            match produce_future.await {
                Ok(delivery) => println!("Sent: {:?}", delivery),
                Err((e, _)) => println!("Error: {:?}", e),
            }
        }
    } else if let Err(e) = bytes {
        println!("Error encoding Avro: {}", e);
        return Err(e.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn produce_message_json(topic: &str, payload: &str, key: &str) -> Result<(), String> {
    let producer = KafkaConnection::get_producer_instance().lock().await;
    if let Some(producer) = &*producer {
        let produce_future = producer.send(
            FutureRecord::to(topic).key(key).payload(payload),
            Duration::from_secs(10),
        );

        return match produce_future.await {
            Ok(delivery) => {
                println!("Sent: {:?}", delivery);
                Ok(())
            }
            Err((e, _)) => {
                let err = format!("Error producing message: {:?}", e);
                println!("{}", err);

                Err(err)
            }
        };
    }

    Err("Kafka connection not established".to_string())
}

fn _convert_json_map_to_avro(
    map: JsonMap<String, JsonValue>,
    schema: &JsonValue,
) -> Result<Vec<(String, Value)>, String> {
    map.into_iter()
        .map(|(k, v)| {
            let avro_val = _convert_json_value_to_avro(k.clone(), v, schema)?;
            Ok((k, avro_val))
        })
        .collect::<Result<Vec<_>, _>>()
}

// will just keep it as a reminder to read docs first and then waste 5 hours on doing something somebody already did
fn _convert_json_value_to_avro(
    key: String,
    value: JsonValue,
    schema: &JsonValue,
) -> Result<Value, String> {
    let (field, field_type) = match schema {
        serde_json::Value::Object(schema_obj) => {
            if let serde_json::Value::Array(fields) = &schema_obj["fields"] {
                let field = fields.iter().find(|item| {
                    if let serde_json::Value::Object(obj) = item {
                        obj.get("name").and_then(|n| n.as_str()) == Some(key.as_str())
                    } else {
                        false
                    }
                });
                {
                    if let Some(field) = field {
                        match field["type"]["type"].clone() {
                            serde_json::Value::String(s) => (&field["type"], s),
                            _ => {
                                if let serde_json::Value::String(s) = field["type"].clone() {
                                    (&field["type"], s)
                                } else if let serde_json::Value::Array(_) = field["type"].clone() {
                                    (&field["type"], "string".to_string())
                                } else {
                                    (&field["type"], "".to_string())
                                }
                            }
                        }
                    } else {
                        (&JsonValue::Null, "".to_string())
                    }
                }
            } else {
                (&JsonValue::Null, "".to_string())
            }
        }
        _ => {
            println!("Not object");
            (&JsonValue::Null, "".to_string())
        }
    };

    // println!("{}, {}", field_type, key);

    match field_type.as_str() {
        "string" => {
            println!("String: {}", value);
            Ok(Value::String(value.to_string()))
        }
        "long" => match value.as_i64() {
            Some(val) => Ok(Value::Long(val)),
            None => {
                let err = format!("Invalid long value {}", key);
                Err(err)
            }
        },
        "int" => match value.as_i64() {
            Some(val) => Ok(Value::Int(val as i32)),
            None => {
                let err = format!("Invalid int value {}", key);
                Err(err)
            }
        },
        "bytes" => Ok(Value::Bytes(Vec::from(value.to_string().as_bytes()))),
        "float" => match value.as_i64() {
            Some(val) => Ok(Value::Float(val as f32)),
            None => {
                let err = format!("Invalid float value {}", key);
                Err(err)
            }
        },
        "double" => match value.as_f64() {
            Some(val) => Ok(Value::Double(val)),
            None => {
                let err = format!("Invalid double value {}", key);
                Err(err)
            }
        },
        "boolean" => match value.as_bool() {
            Some(val) => Ok(Value::Boolean(val)),
            None => {
                let err = format!("Invalid boolean value {}", key);
                Err(err)
            }
        },
        "fixed" => {
            let byte_array = value.as_array();

            match byte_array {
                Some(array) => {
                    let bytes: Vec<u8> = array
                        .iter()
                        .filter_map(|val| {
                            val.as_u64().and_then(|num| {
                                if num <= u8::MAX as u64 {
                                    Some(num as u8)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();

                    Ok(Value::Fixed(array.len(), bytes))
                }
                None => {
                    let err = format!("Invalid fixed value {}", key);
                    Err(err)
                }
            }
        }
        // "enum" => Value::Enum(0, value.to_string()), // Placeholder for enum position
        // "union" => Value::Union(0, Box::new(sub_value)), // Placeholder, needs actual handling
        "array" => {
            let casted_array = value.as_array();
            match casted_array {
                Some(array) => Ok(Value::Array(
                    array
                        .iter()
                        .map(|val| Value::String(val.to_string()))
                        .collect(),
                )),
                None => {
                    let err = format!("Invalid array value {}", key);
                    Err(err)
                }
            }
        }
        // "map" => Value::Map(Default::default()), // Placeholder, needs actual handling
        "record" => match value {
            JsonValue::Object(map) => {
                // println!("{:?} \n\n----- {:?} \n\n-----", map, field);
                if let Ok(result) = _convert_json_map_to_avro(map, field) {
                    println!("{:?}", result);
                    Ok(Value::Record(result))
                } else {
                    let err = format!("Invalid record value {}", key);
                    Err(err)
                }
            }
            _ => return Err("Record type but value ain't no JSON".to_string()),
        },
        // "duration" => Value::Duration(Duration::new(0, 0)), // Placeholder, needs actual handling
        // "decimal" => Value::Decimal(Decimal::new(0, 0)), // Placeholder, needs actual handling
        // "uuid" => Value::Uuid(Uuid::nil()),
        "date" => match value.as_i64() {
            Some(val) => Ok(Value::Date(val as i32)),
            None => {
                let err = format!("Invalid date value {}", key);
                Err(err)
            }
        },
        "time-millis" => match value.as_i64() {
            Some(val) => Ok(Value::TimeMillis(val as i32)),
            None => {
                let err = format!("Invalid double value {}", key);
                Err(err)
            }
        },
        "time-micros" => match value.as_i64() {
            Some(val) => Ok(Value::TimeMicros(val)),
            None => {
                let err = format!("Invalid time-micros value {}", key);
                Err(err)
            }
        },
        "timestamp-millis" => match value.as_i64() {
            Some(val) => Ok(Value::TimestampMillis(val)),
            None => {
                let err = format!("Invalid timestamp-micros value {}", key);
                Err(err)
            }
        },
        "timestamp-micros" => match value.as_i64() {
            Some(val) => Ok(Value::TimestampMicros(val)),
            None => {
                let err = format!("Invalid timestamp-micros value {}", key);
                Err(err)
            }
        },
        "local-timestamp-millis" => match value.as_i64() {
            Some(val) => Ok(Value::LocalTimestampMillis(val)),
            None => {
                let err = format!("Invalid local-timestamp-millis value {}", key);
                Err(err)
            }
        },
        "local-timestamp-micros" => match value.as_i64() {
            Some(val) => Ok(Value::LocalTimestampMicros(val)),
            None => {
                let err = format!("Invalid local-timestamp-millis value {}", key);
                Err(err)
            }
        },
        _ => {
            let err = format!("Unhandled type {} for key {}", field_type, key);
            println!("{}", err);
            Err(err)
        }
    }
}
