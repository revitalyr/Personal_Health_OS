use async_nats::Client;
use futures_util::stream::StreamExt;
use thiserror::Error;
use tracing::{info, error};
use event_model::MedicalEvent;

#[derive(Debug, Error)]
pub enum NatsError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Publish error: {0}")]
    Publish(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct NatsClient {
    client: Client,
}

impl NatsClient {
    pub async fn new(nats_url: &str) -> Result<Self, NatsError> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| NatsError::Connection(e.to_string()))?;

        info!("Document Processor connected to NATS at {}", nats_url);
        
        Ok(Self { client })
    }

    #[allow(dead_code)]
    pub async fn publish_document_processed(&self, document_id: uuid::Uuid) -> Result<(), NatsError> {
        let subject = "documents.processed";
        let payload = serde_json::json!({
            "document_id": document_id,
            "timestamp": chrono::Utc::now(),
            "event_type": "document_processed"
        });
        
        let payload_bytes = serde_json::to_vec(&payload)?;
        
        self.client
            .publish(subject, payload_bytes.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published document processed event for {}", document_id);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn publish_ocr_completed(&self, document_id: uuid::Uuid, job_id: uuid::Uuid, extracted_text: &str) -> Result<(), NatsError> {
        let subject = "ocr.completed";
        let payload = serde_json::json!({
            "document_id": document_id,
            "job_id": job_id,
            "extracted_text": extracted_text,
            "timestamp": chrono::Utc::now(),
            "event_type": "ocr_completed"
        });
        
        let payload_bytes = serde_json::to_vec(&payload)?;
        
        self.client
            .publish(subject, payload_bytes.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published OCR completed event for document {} (job: {})", document_id, job_id);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn publish_dicom_processed(&self, document_id: uuid::Uuid, metadata: serde_json::Value) -> Result<(), NatsError> {
        let subject = "dicom.processed";
        let payload = serde_json::json!({
            "document_id": document_id,
            "metadata": metadata,
            "timestamp": chrono::Utc::now(),
            "event_type": "dicom_processed"
        });
        
        let payload_bytes = serde_json::to_vec(&payload)?;
        
        self.client
            .publish(subject, payload_bytes.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published DICOM processed event for {}", document_id);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn publish_medical_entities_extracted(&self, document_id: uuid::Uuid, entities: Vec<serde_json::Value>) -> Result<(), NatsError> {
        let subject = "entities.extracted";
        let payload = serde_json::json!({
            "document_id": document_id,
            "entities": entities,
            "timestamp": chrono::Utc::now(),
            "event_type": "medical_entities_extracted"
        });
        
        let payload_bytes = serde_json::to_vec(&payload)?;
        
        self.client
            .publish(subject, payload_bytes.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published medical entities extracted event for document {} ({} entities)", document_id, entities.len());
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn publish_manual_entry_created(&self, patient_id: uuid::Uuid, entry_type: &str, event_id: uuid::Uuid) -> Result<(), NatsError> {
        let subject = "manual_entries.created";
        let payload = serde_json::json!({
            "patient_id": patient_id,
            "entry_type": entry_type,
            "event_id": event_id,
            "timestamp": chrono::Utc::now(),
            "event_type": "manual_entry_created"
        });
        
        let payload_bytes = serde_json::to_vec(&payload)?;
        
        self.client
            .publish(subject, payload_bytes.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published manual entry created event for patient {} (type: {})", patient_id, entry_type);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn subscribe_to_document_events<F>(&self, callback: F) -> Result<(), NatsError>
    where
        F: Fn(DocumentEvent) + Send + Sync + 'static,
    {
        let subject = "documents.*";
        let mut subscriber = self.client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::Connection(e.to_string()))?;

        info!("Subscribed to document events");

        // Spawn a task to handle incoming messages
        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                let subject = message.subject.clone();
                let payload = message.payload;
                
                // Parse event type from subject
                let event_type = match subject.as_str() {
                    "documents.processed" => "document_processed",
                    "documents.uploaded" => "document_uploaded",
                    "documents.deleted" => "document_deleted",
                    _ => "unknown",
                };

                match serde_json::from_slice::<serde_json::Value>(&payload) {
                    Ok(data) => {
                        let document_id = data.get("document_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok());

                        if let Some(doc_id) = document_id {
                            let event = DocumentEvent {
                                event_type: event_type.to_string(),
                                document_id: doc_id,
                                data,
                                timestamp: chrono::Utc::now(),
                            };

                            info!("Received document event: {} for {}", event_type, doc_id);
                            callback(event);
                        } else {
                            error!("Invalid document event: missing document_id");
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize document event: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn subscribe_to_ocr_events<F>(&self, callback: F) -> Result<(), NatsError>
    where
        F: Fn(OcrEvent) + Send + Sync + 'static,
    {
        let subject = "ocr.*";
        let mut subscriber = self.client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::Connection(e.to_string()))?;

        info!("Subscribed to OCR events");

        // Spawn a task to handle incoming messages
        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                let subject = message.subject.clone();
                let payload = message.payload;
                
                // Parse event type from subject
                let event_type = match subject.as_str() {
                    "ocr.started" => "ocr_started",
                    "ocr.completed" => "ocr_completed",
                    "ocr.failed" => "ocr_failed",
                    "ocr.progress" => "ocr_progress",
                    _ => "unknown",
                };

                match serde_json::from_slice::<serde_json::Value>(&payload) {
                    Ok(data) => {
                        let job_id = data.get("job_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok());

                        let document_id = data.get("document_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok());

                        if let Some(job_uuid) = job_id {
                            let event = OcrEvent {
                                event_type: event_type.to_string(),
                                job_id: job_uuid,
                                document_id,
                                data,
                                timestamp: chrono::Utc::now(),
                            };

                            info!("Received OCR event: {} for job: {}", event_type, job_uuid);
                            callback(event);
                        } else {
                            error!("Invalid OCR event: missing job_id");
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize OCR event: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn publish_event(&self, event: &MedicalEvent) -> Result<(), NatsError> {
        let subject = format!("events.{:?}", event.event_type);
        let payload_bytes = serde_json::to_vec(event)?;
        
        self.client
            .publish(subject, payload_bytes.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published event: {} for patient: {}", event.id, event.patient_id);
        Ok(())
    }
}

// Event structures
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DocumentEvent {
    pub event_type: String,
    pub document_id: uuid::Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcrEvent {
    pub event_type: String,
    pub job_id: uuid::Uuid,
    pub document_id: Option<uuid::Uuid>,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
