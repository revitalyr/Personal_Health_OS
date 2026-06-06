use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::app::App;
use telemetry::{trace_request, trace_response};

#[derive(Debug, Deserialize)]
pub struct OcrProcessRequest {
    pub languages: Option<Vec<String>>,
    pub extract_entities: Option<bool>,
    pub confidence_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct OcrStatusQuery {
    pub include_progress: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct OcrJobStatus {
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub status: String, // "pending", "processing", "completed", "failed"
    pub progress: f32, // 0.0 to 1.0
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub pages_processed: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
pub struct OcrResult {
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub extracted_text: String,
    pub confidence_score: f32,
    pub language_detected: String,
    pub processing_time_ms: u64,
    pub extracted_entities: Option<Vec<MedicalEntity>>,
    pub pages: Vec<PageResult>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PageResult {
    pub page_number: u32,
    pub text: String,
    pub confidence: f32,
    pub bounding_boxes: Vec<BoundingBox>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct MedicalEntity {
    pub text: String,
    pub entity_type: String, // "medication", "diagnosis", "symptom", "lab_value", "date", "doctor"
    pub confidence: f32,
    pub start_pos: usize,
    pub end_pos: usize,
    pub normalized_value: Option<String>,
}

pub async fn process_document(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
    Json(request): Json<OcrProcessRequest>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/ocr/process/{}", document_id));
    
    // Check if document exists
    let document = app.document_service.get_document(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to get document: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if document.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Start OCR processing
    let job_id = app.document_service.start_ocr_processing_with_options(
        document_id,
        request.languages,
        request.extract_entities.unwrap_or(true),
        request.confidence_threshold.unwrap_or(0.7),
    ).await
        .map_err(|e| {
            tracing::error!("Failed to start OCR processing: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "job_id": job_id,
        "document_id": document_id,
        "status": "processing_started",
        "started_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::ACCEPTED, std::time::Duration::from_millis(25));
    Ok(Json(response))
}

pub async fn get_ocr_status(
    State(app): State<App>,
    Path(job_id): Path<Uuid>,
    Query(_params): Query<OcrStatusQuery>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/ocr/status/{}", job_id));
    
    let status = app.document_service.get_ocr_job_status(job_id).await
        .map_err(|e| {
            tracing::error!("Failed to get OCR status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match status {
        Some(job_status) => {
            let response = serde_json::json!({
                "job_id": job_id,
                "document_id": job_status.document_id,
                "status": job_status.status,
                "progress": job_status.progress,
                "started_at": job_status.started_at,
                "completed_at": job_status.completed_at,
                "error_message": job_status.error_message,
                "pages_processed": job_status.pages_processed,
                "total_pages": job_status.total_pages,
                "estimated_completion": if job_status.status == "processing" {
                    Some(chrono::Utc::now() + chrono::Duration::seconds(
                        ((1.0 - job_status.progress) * 60.0) as i64
                    ))
                } else {
                    None
                }
            });

            trace_response!(StatusCode::OK, std::time::Duration::from_millis(10));
            Ok(Json(response))
        }
        None => {
            trace_response!(StatusCode::NOT_FOUND, std::time::Duration::from_millis(5));
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_ocr_result(
    State(app): State<App>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/ocr/result/{}", job_id));
    
    let result = app.document_service.get_ocr_result(job_id).await
        .map_err(|e| {
            tracing::error!("Failed to get OCR result: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match result {
        Some(ocr_result) => {
            let response = serde_json::json!({
                "job_id": ocr_result.job_id,
                "document_id": ocr_result.document_id,
                "extracted_text": ocr_result.extracted_text,
                "confidence_score": ocr_result.confidence_score,
                "language_detected": ocr_result.language_detected,
                "processing_time_ms": ocr_result.processing_time_ms,
                "extracted_entities": ocr_result.extracted_entities,
                "pages": ocr_result.pages,
                "completed_at": ocr_result.completed_at,
                "summary": {
                    "total_pages": ocr_result.pages.len(),
                    "total_entities": ocr_result.extracted_entities.as_ref().map_or(0, |e| e.len()),
                    "average_confidence": ocr_result.pages.iter()
                        .map(|p| p.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32)
                        .sum::<f32>() / ocr_result.pages.len() as f32
                }
            });

            trace_response!(StatusCode::OK, std::time::Duration::from_millis(15));
            Ok(Json(response))
        }
        None => {
            trace_response!(StatusCode::NOT_FOUND, std::time::Duration::from_millis(5));
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// Batch OCR processing
#[derive(Debug, Deserialize)]
pub struct BatchOcrRequest {
    pub document_ids: Vec<Uuid>,
    pub languages: Option<Vec<String>>,
    pub extract_entities: Option<bool>,
    pub priority: Option<String>, // "low", "normal", "high"
}

pub async fn process_batch_ocr(
    State(app): State<App>,
    Json(request): Json<BatchOcrRequest>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/ocr/process/batch");
    
    // Validate batch size
    if request.document_ids.len() > 100 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let document_ids = request.document_ids.clone();
    
    // Start batch processing
    let batch_job_id = app.document_service.start_batch_ocr_processing(
        document_ids.clone(),
        request.languages,
        request.extract_entities.unwrap_or(true),
        request.priority.unwrap_or("normal".to_string()),
    ).await
        .map_err(|e| {
            tracing::error!("Failed to start batch OCR processing: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "batch_job_id": batch_job_id,
        "total_documents": document_ids.len(),
        "status": "batch_processing_started",
        "started_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::ACCEPTED, std::time::Duration::from_millis(30));
    Ok(Json(response))
}

pub async fn get_batch_ocr_status(
    State(app): State<App>,
    Path(batch_job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/ocr/batch/{}", batch_job_id));
    
    let status = app.document_service.get_batch_ocr_status(batch_job_id).await
        .map_err(|e| {
            tracing::error!("Failed to get batch OCR status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match status {
        Some(batch_status) => {
            let response = serde_json::json!({
                "batch_job_id": batch_job_id,
                "total_documents": batch_status.total_documents,
                "completed_documents": batch_status.completed_documents,
                "failed_documents": batch_status.failed_documents,
                "status": batch_status.status,
                "progress": batch_status.progress,
                "started_at": batch_status.started_at,
                "estimated_completion": batch_status.estimated_completion
            });

            trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
            Ok(Json(response))
        }
        None => {
            trace_response!(StatusCode::NOT_FOUND, std::time::Duration::from_millis(5));
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// OCR quality improvement
#[derive(Debug, Deserialize)]
pub struct OcrCorrectionRequest {
    pub job_id: Uuid,
    pub corrections: Vec<TextCorrection>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TextCorrection {
    pub original_text: String,
    pub corrected_text: String,
    pub page_number: u32,
    pub bounding_box: Option<BoundingBox>,
}

pub async fn apply_ocr_corrections(
    State(app): State<App>,
    Json(request): Json<OcrCorrectionRequest>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/ocr/{}/correct", request.job_id));
    
    // Apply corrections to OCR result
    let corrections_count = request.corrections.len();
    let corrections_value: Result<Vec<Value>, StatusCode> = request.corrections.into_iter()
        .map(|c| serde_json::to_value(c).map_err(|e| {
            tracing::error!("Failed to serialize correction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        .collect();
    
    let corrections_value = corrections_value?;
    
    let corrected_result = app.document_service.apply_ocr_corrections(
        request.job_id,
        corrections_value,
    ).await
        .map_err(|e| {
            tracing::error!("Failed to apply OCR corrections: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "job_id": request.job_id,
        "corrections_applied": corrections_count,
        "corrected_result": corrected_result,
        "corrected_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(40));
    Ok(Json(response))
}

// OCR template management
#[derive(Debug, Deserialize, Serialize)]
pub struct OcrTemplate {
    pub name: String,
    pub document_type: String,
    pub field_mappings: Vec<FieldMapping>,
    pub extraction_rules: Vec<ExtractionRule>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FieldMapping {
    pub field_name: String,
    pub pattern: String,
    pub data_type: String, // "text", "number", "date", "boolean"
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractionRule {
    pub rule_name: String,
    pub pattern: String,
    pub entity_type: String,
    pub confidence_threshold: f32,
}

pub async fn create_ocr_template(
    State(app): State<App>,
    Json(template): Json<OcrTemplate>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/ocr/templates");
    
    let template_value = serde_json::to_value(template)
        .map_err(|e| {
            tracing::error!("Failed to serialize OCR template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let template_id = app.document_service.create_ocr_template(template_value).await
        .map_err(|e| {
            tracing::error!("Failed to create OCR template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "template_id": template_id,
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

pub async fn apply_ocr_template(
    State(app): State<App>,
    Path((document_id, template_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/ocr/{}/apply-template/{}", document_id, template_id));
    
    let extracted_data = app.document_service.apply_ocr_template(document_id, template_id).await
        .map_err(|e| {
            tracing::error!("Failed to apply OCR template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "template_id": template_id,
        "extracted_data": extracted_data,
        "applied_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(50));
    Ok(Json(response))
}
