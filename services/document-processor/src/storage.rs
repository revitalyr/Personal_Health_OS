use crate::config::{Config, StorageType};
use std::path::Path;
use uuid::Uuid;
use tokio::fs;
#[allow(unused_imports)]
use tracing::{info, error, warn};

pub struct DocumentStorage {
    config: Config,
}

impl DocumentStorage {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize storage based on type
        match &config.storage_type {
            StorageType::Local => {
                if let Some(ref path) = config.storage_path {
                    fs::create_dir_all(path).await?;
                    info!("Local storage initialized at: {}", path);
                } else {
                    fs::create_dir_all("./storage").await?;
                    info!("Local storage initialized at: ./storage");
                }
            }
            StorageType::S3 => {
                // TODO: Initialize S3 client
                info!("S3 storage initialized");
            }
            StorageType::Gcs => {
                // TODO: Initialize GCS client
                info!("GCS storage initialized");
            }
        }

        Ok(Self { config })
    }

    pub async fn store_file(&self, filename: &str, data: &[u8]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let storage_path = self.generate_storage_path(filename);
        
        match &self.config.storage_type {
            StorageType::Local => {
                let full_path = self.get_full_path(&storage_path)?;
                fs::write(&full_path, data).await?;
                info!("Stored file locally: {}", full_path.display());
            }
            StorageType::S3 => {
                // TODO: Upload to S3
                warn!("S3 storage not implemented yet");
                return Err("S3 storage not implemented".into());
            }
            StorageType::Gcs => {
                // TODO: Upload to GCS
                warn!("GCS storage not implemented yet");
                return Err("GCS storage not implemented".into());
            }
        }

        Ok(storage_path)
    }

    pub async fn get_file(&self, storage_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        match &self.config.storage_type {
            StorageType::Local => {
                let full_path = self.get_full_path(storage_path)?;
                let data = fs::read(&full_path).await?;
                info!("Retrieved file locally: {}", full_path.display());
                Ok(data)
            }
            StorageType::S3 => {
                // TODO: Download from S3
                warn!("S3 storage not implemented yet");
                Err("S3 storage not implemented".into())
            }
            StorageType::Gcs => {
                // TODO: Download from GCS
                warn!("GCS storage not implemented yet");
                Err("GCS storage not implemented".into())
            }
        }
    }

    pub async fn delete_file(&self, storage_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &self.config.storage_type {
            StorageType::Local => {
                let full_path = self.get_full_path(storage_path)?;
                fs::remove_file(&full_path).await?;
                info!("Deleted file locally: {}", full_path.display());
            }
            StorageType::S3 => {
                // TODO: Delete from S3
                warn!("S3 storage not implemented yet");
            }
            StorageType::Gcs => {
                // TODO: Delete from GCS
                warn!("GCS storage not implemented yet");
            }
        }

        Ok(())
    }

    pub async fn file_exists(&self, storage_path: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match &self.config.storage_type {
            StorageType::Local => {
                let full_path = self.get_full_path(storage_path)?;
                Ok(full_path.exists())
            }
            StorageType::S3 => {
                // TODO: Check S3
                warn!("S3 storage not implemented yet");
                Ok(false)
            }
            StorageType::Gcs => {
                // TODO: Check GCS
                warn!("GCS storage not implemented yet");
                Ok(false)
            }
        }
    }

    pub async fn get_file_size(&self, storage_path: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        match &self.config.storage_type {
            StorageType::Local => {
                let full_path = self.get_full_path(storage_path)?;
                let metadata = fs::metadata(&full_path).await?;
                Ok(metadata.len())
            }
            StorageType::S3 => {
                // TODO: Get S3 file size
                warn!("S3 storage not implemented yet");
                Ok(0)
            }
            StorageType::Gcs => {
                // TODO: Get GCS file size
                warn!("GCS storage not implemented yet");
                Ok(0)
            }
        }
    }

    #[allow(dead_code)]
    pub async fn list_files(&self, prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        match &self.config.storage_type {
            StorageType::Local => {
                let base_path = self.get_base_path()?;
                let search_path = base_path.join(prefix);
                
                let mut files = Vec::new();
                if search_path.exists() {
                    let mut entries = fs::read_dir(&search_path).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(relative_path) = path.strip_prefix(&base_path).ok() {
                                files.push(relative_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
                
                Ok(files)
            }
            StorageType::S3 => {
                // TODO: List S3 files
                warn!("S3 storage not implemented yet");
                Ok(vec![])
            }
            StorageType::Gcs => {
                // TODO: List GCS files
                warn!("GCS storage not implemented yet");
                Ok(vec![])
            }
        }
    }

    // Helper methods
    fn generate_storage_path(&self, filename: &str) -> String {
        let file_id = Uuid::new_v4();
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        format!("documents/{}/{}/{}.{}", 
            chrono::Utc::now().format("%Y/%m/%d"),
            file_id,
            file_id,
            extension
        )
    }

    fn get_full_path(&self, storage_path: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let base_path = self.get_base_path()?;
        Ok(base_path.join(storage_path))
    }

    fn get_base_path(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        match &self.config.storage_type {
            StorageType::Local => {
                let path = self.config.storage_path
                    .as_ref()
                    .map(|p| Path::new(p))
                    .unwrap_or_else(|| Path::new("./storage"));
                Ok(path.to_path_buf())
            }
            StorageType::S3 | StorageType::Gcs => {
                Err("Cloud storage doesn't have local base path".into())
            }
        }
    }

    #[allow(dead_code)]
    pub async fn create_preview(&self, storage_path: &str, preview_size: (u32, u32)) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Get original file
        let original_data = self.get_file(storage_path).await?;
        
        // Generate preview based on file type
        let file_extension = Path::new(storage_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match file_extension.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "tiff" => {
                self.generate_image_preview(&original_data, preview_size).await
            }
            "pdf" => {
                self.generate_pdf_preview(&original_data, preview_size).await
            }
            "dcm" | "dicom" => {
                self.generate_dicom_preview(&original_data, preview_size).await
            }
            _ => {
                // Generate generic preview
                self.generate_generic_preview(&original_data).await
            }
        }
    }

    #[allow(dead_code)]
    async fn generate_image_preview(&self, image_data: &[u8], size: (u32, u32)) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Use image crate to resize image
        let img = image::load_from_memory(image_data)?;
        let resized = img.resize(size.0, size.1, image::imageops::FilterType::Lanczos3);
        
        let mut buffer = Vec::new();
        resized.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;
        
        Ok(buffer)
    }

    #[allow(dead_code)]
    async fn generate_pdf_preview(&self, _pdf_data: &[u8], _size: (u32, u32)) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Generate PDF preview (first page as image)
        warn!("PDF preview generation not implemented yet");
        Ok(vec![])
    }

    #[allow(dead_code)]
    async fn generate_dicom_preview(&self, _dicom_data: &[u8], _size: (u32, u32)) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Generate DICOM preview
        warn!("DICOM preview generation not implemented yet");
        Ok(vec![])
    }

    #[allow(dead_code)]
    async fn generate_generic_preview(&self, _data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Generate a generic file icon preview
        warn!("Generic preview generation not implemented yet");
        Ok(vec![])
    }

    #[allow(dead_code)]
    pub async fn get_file_metadata(&self, storage_path: &str) -> Result<FileMetadata, Box<dyn std::error::Error + Send + Sync>> {
        let size = self.get_file_size(storage_path).await?;
        let exists = self.file_exists(storage_path).await?;
        
        Ok(FileMetadata {
            storage_path: storage_path.to_string(),
            size,
            exists,
            created_at: chrono::Utc::now(), // TODO: Get actual creation time
            modified_at: chrono::Utc::now(), // TODO: Get actual modification time
            content_type: self.guess_content_type(storage_path),
        })
    }

    #[allow(dead_code)]
    fn guess_content_type(&self, storage_path: &str) -> String {
        let extension = Path::new(storage_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "pdf" => "application/pdf".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "png" => "image/png".to_string(),
            "tiff" => "image/tiff".to_string(),
            "dcm" | "dicom" => "application/dicom".to_string(),
            "txt" => "text/plain".to_string(),
            "doc" => "application/msword".to_string(),
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub storage_path: String,
    pub size: u64,
    pub exists: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub content_type: String,
}
