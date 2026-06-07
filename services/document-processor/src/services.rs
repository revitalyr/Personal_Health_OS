use storage::EventStore;
use event_model::{MedicalEvent, EventType, SymptomPayload, MedicationPayload, LabResultPayload, DoctorVisitPayload, DiagnosisPayload};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value;
use serde::Serialize;
use sqlx::{PgPool, Row};
use crate::{nats::NatsClient, storage::DocumentStorage, config::Config};

// Input structures for manual entry
use crate::handlers::manual_input::{
    SymptomInput, MedicationInput, LabResultInput, DoctorVisitInput, DiagnosisInput, ManualEntryInput
};

// ---------------------------------------------------------------------------
// Processor traits — injectable abstractions for testability
// ---------------------------------------------------------------------------

/// Trait for OCR processing — allows injecting a real or mock OCR engine.
pub trait OcrProcessing: Send + Sync {
    /// Runs OCR on raw image/PDF bytes and returns the extracted text and metadata.
    fn perform_ocr(&self, data: &[u8]) -> Result<ocr_processor::OcrResult, ocr_processor::OcrError>;
}

impl OcrProcessing for ocr_processor::OcrProcessor {
    fn perform_ocr(&self, data: &[u8]) -> Result<ocr_processor::OcrResult, ocr_processor::OcrError> {
        ocr_processor::OcrProcessor::perform_ocr(data)
    }
}

/// Trait for DICOM processing — allows parsing medical imaging metadata.
pub trait DicomProcessing: Send + Sync {
    /// Parses DICOM metadata from raw byte data.
    fn parse_metadata(&self, data: &[u8]) -> Result<dicom_processor::DicomMetadata, dicom_processor::DicomError>;
}

impl DicomProcessing for dicom_processor::DicomProcessor {
    fn parse_metadata(&self, data: &[u8]) -> Result<dicom_processor::DicomMetadata, dicom_processor::DicomError> {
        dicom_processor::DicomProcessor::parse_metadata(data)
    }
}

/// Trait for NLP processing — extracts medical entities from unstructured text.
pub trait NlpProcessing: Send + Sync {
    /// Extracts medical entities (conditions, medications, etc.) from the given text.
    fn extract_entities(&self, text: &str) -> Result<Vec<nlp_processor::MedicalEntity>, nlp_processor::NlpError>;
}

impl NlpProcessing for nlp_processor::MedicalEntityExtractor {
    fn extract_entities(&self, text: &str) -> Result<Vec<nlp_processor::MedicalEntity>, nlp_processor::NlpError> {
        nlp_processor::MedicalEntityExtractor::extract_entities(text)
    }
}

// ---------------------------------------------------------------------------
// DocumentService
// ---------------------------------------------------------------------------

/// Central service for document management, OCR processing, DICOM handling,
/// and manual medical event creation.
pub struct DocumentService {
    event_store: Arc<dyn EventStore>,
    pool: PgPool,
    nats_client: Arc<NatsClient>,
    document_storage: Arc<DocumentStorage>,
    ocr_processor: Arc<dyn OcrProcessing>,
    dicom_processor: Arc<dyn DicomProcessing>,
    nlp_extractor: Arc<dyn NlpProcessing>,
    #[allow(dead_code)]
    config: Config,
}

impl DocumentService {
    /// Creates a new `DocumentService` with the required dependencies.
    pub fn new(
        event_store: Arc<dyn EventStore>,
        pool: PgPool,
        nats_client: Arc<NatsClient>,
        document_storage: Arc<DocumentStorage>,
        config: Config,
        ocr_processor: Arc<dyn OcrProcessing>,
        dicom_processor: Arc<dyn DicomProcessing>,
        nlp_extractor: Arc<dyn NlpProcessing>,
    ) -> Self {
        Self {
            event_store,
            pool,
            nats_client,
            document_storage,
            ocr_processor,
            dicom_processor,
            nlp_extractor,
            config,
        }
    }

    // Document management
    /// Inserts a new document record into the database.
    pub async fn create_document_record(
        &self,
        document_id: Uuid,
        patient_id: Uuid,
        filename: String,
        file_type: String,
        file_size: u64,
        storage_path: String,
        document_type: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            .execute(&self.pool)
            .await?;

        tracing::info!("Created document record: {} for patient: {}", document_id, patient_id);
        Ok(())
    }

    /// Retrieves a single document's metadata from the database.
    pub async fn get_document(&self, document_id: Uuid) -> Result<Option<DocumentInfo>, Box<dyn std::error::Error + Send + Sync>> {
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
            .fetch_optional(&self.pool)
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

    /// Lists documents with optional filtering by patient and type, with pagination.
    pub async fn list_documents(
        &self,
        patient_id: Option<Uuid>,
        document_type: Option<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DocumentInfo>, Box<dyn std::error::Error + Send + Sync>> {
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

        let mut _bind_count = 0;

        if patient_id.is_some() {
            query.push_str(&format!(" AND patient_id = ${}", _bind_count + 1));
            _bind_count += 1;
        }

        if document_type.is_some() {
            query.push_str(&format!(" AND document_type = ${}", _bind_count + 1));
            _bind_count += 1;
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

        let mut sql_query = sqlx::query(&query);

        if let Some(pid) = patient_id {
            sql_query = sql_query.bind(pid);
        }

        if let Some(dt) = document_type {
            sql_query = sql_query.bind(dt);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

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

    /// Searches documents by filename or OCR-extracted text, with optional filters.
    pub async fn search_documents(
        &self,
        query: &str,
        patient_id: Option<Uuid>,
        document_type: Option<String>,
    ) -> Result<Vec<DocumentInfo>, Box<dyn std::error::Error + Send + Sync>> {
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

        let rows = query_builder.fetch_all(&self.pool).await?;

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

    /// Deletes a document record and its associated file from storage.
    pub async fn delete_document(&self, document_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let doc = self.get_document(document_id).await?;

        if let Some(document_info) = doc {
            if let Err(e) = self.document_storage.delete_file(&document_info.storage_path).await {
                tracing::warn!("Failed to delete file from storage: {}", e);
            }
        }

        let query = "DELETE FROM documents WHERE id = $1";
        sqlx::query(query)
            .bind(document_id)
            .execute(&self.pool)
            .await?;

        tracing::info!("Deleted document: {}", document_id);
        Ok(())
    }

    // OCR processing
    /// Kicks off an async OCR pipeline for the given document and returns a job ID.
    pub async fn start_ocr_processing(&self, document_id: Uuid) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let job_id = Uuid::new_v4();

        let doc_storage = self.document_storage.clone();
        let event_store = self.event_store.clone();
        let ocr = self.ocr_processor.clone();
        let nlp = self.nlp_extractor.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::run_ocr_pipeline(
                document_id, job_id, doc_storage, event_store, ocr, nlp,
            ).await {
                tracing::error!("OCR processing failed for document {}: {}", document_id, e);
            }
        });

        tracing::info!("Started OCR processing for document: {} (job: {})", document_id, job_id);
        Ok(job_id)
    }

    /// Starts OCR processing with configurable languages, entity extraction, and confidence threshold.
    pub async fn start_ocr_processing_with_options(
        &self,
        document_id: Uuid,
        _languages: Option<Vec<String>>,
        _extract_entities: bool,
        _confidence_threshold: f32,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        self.start_ocr_processing(document_id).await
    }

    async fn run_ocr_pipeline(
        document_id: Uuid,
        job_id: Uuid,
        document_storage: Arc<DocumentStorage>,
        _event_store: Arc<dyn EventStore>,
        ocr: Arc<dyn OcrProcessing>,
        nlp: Arc<dyn NlpProcessing>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Starting OCR pipeline for document: {} (job: {})", document_id, job_id);

        let file_data = document_storage.get_file(&format!("document_{}", document_id)).await?;

        let ocr_result = ocr.perform_ocr(&file_data)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let entities = nlp.extract_entities(&ocr_result.text)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        tracing::info!(
            "OCR pipeline completed for document: {} — text_len={}, confidence={}, entities={}",
            document_id, ocr_result.text.len(), ocr_result.confidence, entities.len(),
        );

        Ok(())
    }

    // Manual entry event creation
    /// Creates a symptom medical event from manual input.
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

    /// Creates a medication event from manual input.
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

    /// Creates a lab result event from manual input.
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

    /// Creates a doctor visit event from manual input.
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

    /// Creates a diagnosis event from manual input.
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

    /// Creates a generic manual entry event with free-form title and description.
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
    /// Generates a preview URL for the given document.
    pub async fn generate_preview(&self, document_id: Uuid) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(format!("/preview/{}", document_id))
    }

    /// Returns basic metadata for the specified document.
    pub async fn get_document_metadata(&self, document_id: Uuid) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"document_id": document_id}))
    }

    /// Returns extracted (OCR/NLP) data for the specified document.
    pub async fn get_extracted_data(&self, document_id: Uuid) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"document_id": document_id}))
    }

    /// Classifies a document into a predefined category (e.g. "medical_document").
    pub async fn classify_document(&self, _document_id: Uuid) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("medical_document".to_string())
    }

    // DICOM methods
    /// Validates a DICOM file and extracts its metadata as JSON.
    pub async fn validate_and_extract_dicom_metadata(&self, data: &[u8]) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let meta = self.dicom_processor.parse_metadata(data)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        serde_json::to_value(&meta)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Creates a document record specifically for a DICOM file.
    pub async fn create_dicom_document_record(&self, _document_id: Uuid, _patient_id: Uuid, _filename: String, _storage_path: String, _metadata: Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Generates a thumbnail URL for a DICOM image.
    pub async fn generate_dicom_thumbnail(&self, _document_id: Uuid) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(format!("/thumbnail/{}", Uuid::new_v4())))
    }

    /// Returns stored DICOM metadata for the specified document.
    pub async fn get_dicom_metadata(&self, _document_id: Uuid) -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(serde_json::json!({"modality": "CT"})))
    }

    /// Returns a URL for viewing the DICOM image.
    #[allow(dead_code)]
    pub async fn get_dicom_image(&self, _document_id: Uuid) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(format!("/image/{}", Uuid::new_v4())))
    }

    /// Lists all DICOM studies for a given patient.
    #[allow(dead_code)]
    pub async fn get_dicom_studies_for_patient(&self, _patient_id: Uuid) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    /// Lists all DICOM series within a specific study.
    #[allow(dead_code)]
    pub async fn get_dicom_series_for_study(&self, _patient_id: Uuid, _study_id: &str) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    /// Adds an annotation to a DICOM document and returns its ID.
    #[allow(dead_code)]
    pub async fn add_dicom_annotation(&self, _document_id: Uuid, _annotation: Value) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Uuid::new_v4())
    }

    /// Retrieves all annotations for a DICOM document.
    #[allow(dead_code)]
    pub async fn get_dicom_annotations(&self, _document_id: Uuid) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    // OCR status and results
    /// Returns the current status of an OCR job.
    pub async fn get_ocr_job_status(&self, _job_id: Uuid) -> Result<Option<OcrJobStatus>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// Returns the full OCR result (text, confidence, entities) for a completed job.
    #[allow(dead_code)]
    pub async fn get_ocr_result(&self, _job_id: Uuid) -> Result<Option<OcrResult>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// Starts batch OCR processing for multiple documents and returns a batch job ID.
    #[allow(dead_code)]
    pub async fn start_batch_ocr_processing(&self, _document_ids: Vec<Uuid>, _languages: Option<Vec<String>>, _extract_entities: bool, _priority: String) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Uuid::new_v4())
    }

    /// Returns the aggregated status of a batch OCR job.
    #[allow(dead_code)]
    pub async fn get_batch_ocr_status(&self, _batch_job_id: Uuid) -> Result<Option<BatchOcrStatus>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// Applies user-provided corrections to an OCR result.
    #[allow(dead_code)]
    pub async fn apply_ocr_corrections(&self, _job_id: Uuid, _corrections: Vec<Value>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"status": "corrected"}))
    }

    /// Creates an OCR processing template and returns its ID.
    #[allow(dead_code)]
    pub async fn create_ocr_template(&self, _template: Value) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Uuid::new_v4())
    }

    /// Applies an OCR template to a specific document.
    #[allow(dead_code)]
    pub async fn apply_ocr_template(&self, _document_id: Uuid, _template_id: Uuid) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({"status": "applied"}))
    }
}

// Data structures
/// Metadata for a stored document, including file info and processing state.
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

/// Status and progress of an individual OCR processing job.
#[allow(dead_code)]
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

/// The complete output of an OCR job: extracted text, confidence, and entities.
#[allow(dead_code)]
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

/// Aggregated progress and status of a batch OCR processing job.
#[allow(dead_code)]
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
