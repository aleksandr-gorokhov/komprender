use once_cell::sync::Lazy;
use schema_registry_converter::async_impl::schema_registry::{
    get_all_subjects, get_schema_by_subject, SrSettings,
};
use schema_registry_converter::schema_registry_common::SubjectNameStrategy::RecordNameStrategy;
use tokio::sync::Mutex;
use tokio::time::Duration;

pub struct SchemaRegistry;

pub static SCHEMA_REGISTRY_SETTINGS: Lazy<Mutex<Option<SrSettings>>> =
    Lazy::new(|| Mutex::new(None));

impl SchemaRegistry {
    pub async fn connect(url: &str) -> Result<bool, String> {
        let url = if url.is_empty() {
            return Ok(false);
        } else {
            url.to_string()
        };
        println!("Connecting to Schema Registry: {}", url);

        let sr_settings = SrSettings::new_builder(String::from(url))
            .set_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        if let Ok(_) = get_all_subjects(&sr_settings).await {
            println!("Connected to Schema Registry");
            let mut sr_guard = SchemaRegistry::get_settings().lock().await;
            match &*sr_guard {
                Some(_) => {}
                None => {
                    *sr_guard = Some(sr_settings);
                }
            }
        } else {
            return Err("Could not connect to Schema Registry".to_string());
        }

        Ok(true)
    }

    pub async fn get_all_subjects() -> Result<Vec<String>, String> {
        println!("Getting all subjects");
        let sr_guard = SchemaRegistry::get_settings().lock().await;
        println!("Guard locked");

        if let Some(sr_settings) = &*sr_guard {
            println!("Got guard");

            let subjects = get_all_subjects(sr_settings)
                .await
                .map_err(|e| e.to_string());
            subjects
        } else {
            println!("Got nothing");

            return Err("Schema Registry not connected".to_string());
        }
    }

    pub async fn get_schema(schema_name: &str) -> Result<String, String> {
        let strategy = RecordNameStrategy(schema_name.to_string());
        let sr_guard = SchemaRegistry::get_settings().lock().await;
        return if let Some(sr_settings) = &*sr_guard {
            let subject = get_schema_by_subject(sr_settings, &strategy)
                .await
                .map_err(|e| e.to_string());
            if let Ok(subject) = subject {
                Ok(subject.schema)
            } else {
                Err("Could not get schema".to_string())
            }
        } else {
            Err("Schema Registry not connected".to_string())
        };
    }

    pub async fn disconnect() -> Result<(), String> {
        let mut sr_guard = SchemaRegistry::get_settings().lock().await;
        *sr_guard = None;
        Ok(())
    }

    pub fn get_settings() -> &'static Mutex<Option<SrSettings>> {
        &SCHEMA_REGISTRY_SETTINGS
    }
}

#[tauri::command]
pub async fn fetch_sr_subjects() -> Result<Vec<String>, String> {
    SchemaRegistry::get_all_subjects().await
}

#[tauri::command]
pub async fn fetch_schema(subject: &str) -> Result<String, String> {
    SchemaRegistry::get_schema(subject).await
}
