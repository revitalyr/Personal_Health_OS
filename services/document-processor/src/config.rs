use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub nats_url: String,
    pub storage_type: StorageType,
    pub storage_path: Option<String>,
    pub aws_region: Option<String>,
    pub aws_bucket: Option<String>,
    pub ocr_languages: Vec<String>,
    pub max_file_size: usize,
    pub supported_formats: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    Local,
    S3,
    Gcs,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8082".to_string())
            .parse()
            .unwrap_or(8082);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string());

        let nats_url = env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());

        let storage_type_str = env::var("STORAGE_TYPE")
            .unwrap_or_else(|_| "local".to_string());
        
        let storage_type = match storage_type_str.as_str() {
            "s3" => StorageType::S3,
            "gcs" => StorageType::Gcs,
            _ => StorageType::Local,
        };

        let storage_path = env::var("STORAGE_PATH").ok();

        let aws_region = env::var("AWS_REGION").ok();
        let aws_bucket = env::var("AWS_BUCKET").ok();

        let ocr_languages = env::var("OCR_LANGUAGES")
            .unwrap_or_else(|_| "rus,eng".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let max_file_size: usize = env::var("MAX_FILE_SIZE")
            .unwrap_or_else(|_| "50MB".to_string())
            .parse()
            .unwrap_or(50 * 1024 * 1024);

        let supported_formats = env::var("SUPPORTED_FORMATS")
            .unwrap_or_else(|_| {
                "pdf,jpg,jpeg,png,tiff,dicom,doc,docx,txt".to_string()
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Config {
            port,
            database_url,
            nats_url,
            storage_type,
            storage_path,
            aws_region,
            aws_bucket,
            ocr_languages,
            max_file_size,
            supported_formats,
        })
    }
}
