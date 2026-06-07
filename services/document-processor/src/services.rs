use storage::{EventStore, Result as StorageResult};
use event_model::{MedicalEvent, EventType, SymptomPayload, MedicationPayload, LabResultPayload, DoctorVisitPayload, DiagnosisPayload};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value;
use serde::Serialize;
use sqlx::Row;
use crate::{nats::NatsClient, storage::DocumentStorage, config::Config, processors::{OcrProcessor, DicomProcessor, MedicalEntityExtractor}};

// Input structures for manual entry
use crate::handlers::manual_input::{
    SymptomInput, MedicationInput, LabResultInput, DoctorVisitInput, DiagnosisInput, ManualEntryInput
};

pub struct DocumentService {
    event_store: Arc<EventStore>,
    nats_client: Arc<NatsClient>,
    document_storage: Arc<DocumentStorage>,
    ocr_processor: Arc<OcrProcessor>,
    dicom_processor: Arc<DicomProcessor>,
    entity_extractor: Arc<MedicalEntityExtractor>,
    config: Config,
}

impl DocumentService {
    pub fn new(
        event_store: Arc<EventStore>,
        nats_client: Arc<NatsClient>,
        document_storage: Arc<DocumentStorage>,
        config: Config,
    ) -> Self {
        let ocr_processor = Arc::new(OcrProcessor::new(config.clone()));
        let dicom_processor = Arc::new(DicomProcessor::new(config.clone()));
        let entity_extractor = Arc::new(MedicalEntityExtractor::new(config.clone()));

        Self {
            event_store,
            nats_client,
            document_storage,
            ocr_processor,
            dicom_processor,
            entity_extractor,
            config,
        }
    }

    // Document management
    pub async fn create_document_record(
        &self,
        document_id: Uuid,
        patient_id: Uuid,
        filename: String,
        file_type: String,
        file_size: u64,
        storage_path: String,
        document_type: Option<String>,
    ) -> StorageResult<()> {
        let query = r#"
            INSERT INTO documents (
                id, patient_id, filename, file_type, file_size, 
                storage_path, document_type, processed, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        sqlx::query(query)
            .bind(document_id)
            .bind(patient_id)
            .bind(filename)
            .bind(file_type)
            .bind(file_size as i64)
            .bind(storage_path)
            .bind(document_type)
            .bind(false)
            .bind(Utc::now())
            .execute(self.event_store.pool())
            .await?;

        tracing::info!("Created document record: {} for patient: {}", document_id, patient_id);
        Ok(())
    }

    pub async fn get_document(&self, document_id: Uuid) -> StorageResult<Option<DocumentInfo>> {
        let query = r#"
            SELECT id, patient_id, filename, file_type, file_size, 
                   storage_path, document_type, facility, document_date,
                   processed, ocr_extracted_text, extracted_data,
                   created_at, updated_at
            FROM documents 
            WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(document_id)
            .fetch_optional(self.event_store.pool())
            .await?;

        if let Some(row) = row {
            let doc = DocumentInfo {
                id: row.get("id"),
                patient_id: row.get("patient_id"),
                filename: row.get("filename"),
                file_type: row.get("file_type"),
                file_size: row.get::<i64, _>("file_size") as u64,
                storage_path: row.get("storage_path"),
                document_type: row.get("document_type"),
                facility: row.get("facility"),
                document_date: row.get("document_date"),
                processed: row.get("processed"),
                ocr_extracted_text: row.get("ocr_extracted_text"),
                extracted_data: row.get("extracted_data"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(doc))
        } else {
            Ok(None)
        }
    }

    pub async fn list_documents(
        &self,
        patient_id: Option<Uuid>,
        document_type: Option<String>,
        limit: usize,
        offset: usize,
    ) -> StorageResult<Vec<DocumentInfo>> {
        let mut query = String::from(
            r#"
            SELECT id, patient_id, filename, file_type, file_size, 
                   storage_path, document_type, facility, document_date,
                   processed, ocr_extracted_text, extracted_data,
                   created_at, updated_at
            FROM documents 
            WHERE 1=1
        "#
        );

        let mut bind_count = 0;

        if patient_id.is_some() {
            query.push_str(&format!(" AND patient_id = ${}", bind_count + 1));
            bind_count += 1;
        }

        if document_type.is_some() {
            query.push_str(&format!(" AND document_type = ${}", bind_count + 1));
            bind_count += 1;
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

        let mut sql_query = sqlx::query(&query);

        if let Some(pid) = patient_id {
            sql_query = sql_query.bind(pid);
        }

        if let Some(dt) = document_type {
            sql_query = sql_query.bind(dt);
        }

        let rows = sql_query.fetch_all(self.event_store.pool()).await?;

        let mut documents = Vec::new();
        for row in rows {
            let doc = DocumentInfo {
                id: row.get("id"),
                patient_id: row.get("patient_id"),
                filename: row.get("filename"),
                file_type: row.get("file_type"),
                file_size: row.get::<i64, _>("file_size") as u64,
                storage_path: row.get("storage_path"),
                document_type: row.get("document_type"),
                facility: row.get("facility"),
                document_date: row.get("document_date"),
                processed: row.get("processed"),
                ocr_extracted_text: row.get("ocr_extracted_text"),
                extracted_data: row.get("extracted_data"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            documents.push(doc);
        }

        Ok(documents)
    }

    pub async fn search_documents(
        &self,
        query: &str,
        patient_id: Option<Uuid>,
        document_type: Option<String>,
    ) -> StorageResult<Vec<DocumentInfo>> {
        let search_query = format!("%{}%", query);
        let mut sql_query = String::from(
            r#"
            SELECT id, patient_id, filename, file_type, file_size, 
                   storage_path, document_type, facility, document_date,
                   processed, ocr_extracted_text, extracted_data,
                   created_at, updated_at
            FROM documents 
            WHERE (filename ILIKE $1 OR ocr_extracted_text ILIKE $1)
        "#
        );

        let mut bind_count = 1;

        if patient_id.is_some() {
            bind_count += 1;
            sql_query.push_str(&format!(" AND patient_id = ${}", bind_count));
        }

        if document_type.is_some() {
            bind_count += 1;
            sql_query.push_str(&format!(" AND document_type = ${}", bind_count));
        }

        sql_query.push_str(" ORDER BY created_at DESC LIMIT 50");

        let mut query_builder = sqlx::query(&sql_query).bind(&search_query);

        if let Some(pid) = patient_id {
            query_builder = query_builder.bind(pid);
        }

        if let Some(dt) = document_type {
            query_builder = query_builder.bind(dt);
        }

        let rows = query_builder.fetch_all(self.event_store.pool()).await?;

        let mut documents = Vec::new();
        for row in rows {
            let doc = DocumentInfo {
                id: row.get("id"),
                patient_id: row.get("patient_id"),
                filename: row.get("filename"),
                file_type: row.get("file_type"),
                file_size: row.get::<i64, _>("file_size") as u64,
                storage_path: row.get("storage_path"),
                document_type: row.get("document_type"),
                facility: row.get("facility"),
                document_date: row.get("document_date"),
                processed: row.get("processed"),
                ocr_extracted_text: row.get("ocr_extracted_text"),
                extracted_data: row.get("extracted_data"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            documents.push(doc);
        }

        Ok(documents)
    }

    pub async fn delete_document(&self, document_id: Uuid) -> StorageResult<()> {
        // Get document info for cleanup
        let doc = self.get_document(document_id).await?;
        
        if let Some(document_info) = doc {
            // Delete from storage
            if let Err(e) = self.document_storage.delete_file(&document_info.storage_path).await {
                tracing::warn!("Failed to delete file from storage: {}", e);
            }
        }

        // Delete from database
        let query = "DELETE FROM documents WHERE id = $1";
        sqlx::query(query)
            .bind(document_id)
            .execute(self.event_store.pool())
            .await?;

        tracing::info!("Deleted document: {}", document_id);
        Ok(())
    }

    // OCR processing
    pub async fn start_ocr_processing(&self, document_id: Uuid) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let job_id = Uuid::new_v4();
        
        // Start OCR processing in background
        let ocr_processor = self.ocr_processor.clone();
        let document_storage = self.document_storage.clone();
        let event_store = self.event_store.clone();
        
        tokio::spawn(async move {
            if let Err(e) = ocr_processor.process_document(document_id, job_id, document_storage, event_store).await {
                tracing::error!("OCR processing failed for document {}: {}", document_id, e);
            }
        });

        tracing::info!("Started OCR processing for document: {} (job: {})", document_id, job_id);
        Ok(job_id)
    }

    pub async fn start_ocr_processing_with_options(
        &self,
        document_id: Uuid,
        languages: Option<Vec<String>>,
        extract_entities: bool,
        confidence_threshold: f32,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let job_id = Uuid::new_v4();
        
        // Start OCR processing with options
        let ocr_processor = self.ocr_processor.clone();
        let document_storage = self.document_storage.clone();
        let event_store = self.event_store.clone();
        
        tokio::spawn(async move {
            if let Err(e) = ocr_processor.process_document_with_options(
                document_id, 
                job_id, 
                document_storage, 
                event_store,
                languages,
                extract_entities,
                confidence_threshold,
            ).await {
                tracing::error!("OCR processing failed for document {}: {}", document_id, e);
            }
        });

        tracing::info!("Started OCR processing with options for document: {} (job: {})", document_id, job_id);
        Ok(job_id)
    }

    // Manual entry event creation
    pub async fn create_symptom_event(&self, input: SymptomInput) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let payload = SymptomPayload {
            name: input.name.clone(),
            severity: input.severity,
            description: input.description,
            duration: input.duration,
        };

        let event = MedicalEvent::new(
            input.patient_id,
            EventType::SymptomCreated,
            serde_json::to_value(payload)?,
            "manual_input".to_string(),
        );

        self.event_store.store_event(&event).await?;
        self.nats_client.publish_event(&event).await?;

        tracing::info!("Created symptom event: {} for patient: {}", event.id, event.patient_id);
        Ok(event.id)
    }

    pub async fn create_medication_event(&self, input: MedicationInput) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let payload = MedicationPayload {
            name: input.name.clone(),
            dosage: input.dosage.clone(),
            frequency: input.frequency.clone(),
            start_date: input.start_date,
            end_date: input.end_date,
            prescribed_by: input.prescribed_by,
        };

        let event = MedicalEvent::new(
            input.patient_id,
            EventType::MedicationStarted,
            serde_json::to_value(payload)?,
            "manual_input".to_string(),
        );

        self.event_store.store_event(&event).await?;
        self.nats_client.publish_event(&event).await?;

        tracing::info!("Created medication event: {} for patient: {}", event.id, event.patient_id);
        Ok(event.id)
    }

    pub async fn create_lab_result_event(&self, input: LabResultInput) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let payload = LabResultPayload {
            test_name: input.test_name.clone(),
            value: input.value.clone(),
            unit: input.unit.clone(),
            reference_range: input.reference_range,
            status: input.status.clone(),
            facility: input.facility.clone(),
        };

        let event = MedicalEvent::with_timestamp(
            input.patient_id,
            EventType::LabResultReceived,
            serde_json::to_value(payload)?,
            "manual_input".to_string(),
            input.test_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| input.test_date.and_hms_opt(0, 0, 0).unwrap()).and_utc(),
        );

        self.event_store.store_event(&event).await?;
        self.nats_client.publish_event(&event).await?;

        tracing::info!("Created lab result event: {} for patient: {}", event.id, event.patient_id);
        Ok(event.id)
    }

    pub async fn create_doctor_visit_event(&self, input: DoctorVisitInput) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let payload = DoctorVisitPayload {
            doctor_name: input.doctor_name.clone(),
            specialty: input.specialty.clone(),
            facility: input.facility.clone(),
            reason: input.reason.clone(),
            notes: input.notes,
            follow_up_date: input.follow_up_date,
        };

        let event = MedicalEvent::new(
            input.patient_id,
            EventType::DoctorVisit,
            serde_json::to_value(payload)?,
            "manual_input".to_string(),
        );

        self.event_store.store_event(&event).await?;
        self.nats_client.publish_event(&event).await?;

        tracing::info!("Created doctor visit event: {} for patient: {}", event.id, event.patient_id);
        Ok(event.id)
    }

    pub async fn create_diagnosis_event(&self, input: DiagnosisInput) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let payload = DiagnosisPayload {
            condition: input.condition.clone(),
            icd10_code: input.icd10_code,
            diagnosed_by: input.diagnosed_by.clone(),
            severity: input.severity,
            acute: input.acute,
            notes: input.notes,
        };

        let event = MedicalEvent::with_timestamp(
            input.patient_id,
            EventType::Diagnosis,
            serde_json::to_value(payload)?,
            "manual_input".to_string(),
            input.diagnosis_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| input.diagnosis_date.and_hms_opt(0, 0, 0).unwrap()).and_utc(),
        );

        self.event_store.store_event(&event).await?;
        self.nats_client.publish_event(&event).await?;

        tracing::info!("Created diagnosis event: {} for patient: {}", event.id, event.patient_id);
        Ok(event.id)
    }

    pub async fn create_manual_entry_event(&self, input: ManualEntryInput) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "title": input.title,
            "description": input.description,
            "category": input.category,
            "tags": input.tags,
            "attachments": input.attachments,
            "importance": input.importance,
        });

        let event = MedicalEvent::new(
            input.patient_id,
            EventType::DocumentUploaded,
            payload,
            "manual_input".to_string(),
        );

        self.event_store.store_event(&event).await?;
        self.nats_client.publish_event(&event).await?;

        tracing::info!("Created manual entry event: {} for patient: {}", event.id, event.patient_id);
        Ok(event.id)
    }

    // Placeholder methods for other functionality
    pub async fn generate_preview(&self, document_id: Uuid) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(format!("/preview/{}", document_id))
    }

    pub async fn get_document_metadata(&self, document_id: Uuid) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"document_id": document_id}))
    }

    pub async fn get_extracted_data(&self, document_id: Uuid) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"document_id": document_id}))
    }

    pub async fn classify_document(&self, document_id: Uuid) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("medical_document".to_string())
    }

    // DICOM methods
    pub async fn validate_and_extract_dicom_metadata(&self, _data: &[u8]) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"modality": "CT", "study_date": "20240101"}))
    }

    pub async fn create_dicom_document_record(&self, _document_id: Uuid, _patient_id: Uuid, _filename: String, _storage_path: String, _metadata: Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    pub async fn generate_dicom_thumbnail(&self, _document_id: Uuid) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(format!("/thumbnail/{}", Uuid::new_v4())))
    }

    pub async fn get_dicom_metadata(&self, _document_id: Uuid) -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(serde_json::json!({"modality": "CT"})))
    }

    pub async fn get_dicom_image(&self, _document_id: Uuid) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(format!("/image/{}", Uuid::new_v4())))
    }

    pub async fn get_dicom_studies_for_patient(&self, _patient_id: Uuid) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    pub async fn get_dicom_series_for_study(&self, _patient_id: Uuid, _study_id: &str) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    pub async fn add_dicom_annotation(&self, _document_id: Uuid, _annotation: Value) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Uuid::new_v4())
    }

    pub async fn get_dicom_annotations(&self, _document_id: Uuid) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    // OCR status and results
    pub async fn get_ocr_job_status(&self, _job_id: Uuid) -> Result<Option<OcrJobStatus>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    pub async fn get_ocr_result(&self, _job_id: Uuid) -> Result<Option<OcrResult>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    pub async fn start_batch_ocr_processing(&self, _document_ids: Vec<Uuid>, _languages: Option<Vec<String>>, _extract_entities: bool, _priority: String) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Uuid::new_v4())
    }

    pub async fn get_batch_ocr_status(&self, _batch_job_id: Uuid) -> Result<Option<BatchOcrStatus>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    pub async fn apply_ocr_corrections(&self, _job_id: Uuid, _corrections: Vec<Value>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"status": "corrected"}))
    }

    pub async fn create_ocr_template(&self, _template: Value) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Uuid::new_v4())
    }

    pub async fn apply_ocr_template(&self, _document_id: Uuid, _template_id: Uuid) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"status": "applied"}))
    }
}

// Data structures
#[derive(Debug, Clone, Serialize)]
pub struct DocumentInfo {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub file_size: u64,
    pub storage_path: String,
    pub document_type: Option<String>,
    pub facility: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub processed: bool,
    pub ocr_extracted_text: Option<String>,
    pub extracted_data: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct OcrJobStatus {
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub status: String,
    pub progress: f32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub pages_processed: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone)]
pub struct OcrResult {
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub extracted_text: String,
    pub confidence_score: f32,
    pub language_detected: String,
    pub processing_time_ms: u64,
    pub extracted_entities: Option<Vec<Value>>,
    pub pages: Vec<Value>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BatchOcrStatus {
    pub batch_job_id: Uuid,
    pub total_documents: usize,
    pub completed_documents: usize,
    pub failed_documents: usize,
    pub status: String,
    pub progress: f32,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}
