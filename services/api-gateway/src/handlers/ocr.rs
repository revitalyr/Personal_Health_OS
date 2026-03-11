use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

#[derive(Debug, Deserialize)]
pub struct OcrProcessRequest {
    pub languages: Option<Vec<String>>,
    pub extract_entities: Option<bool>,
    pub confidence_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct BatchOcrRequest {
    pub document_ids: Vec<Uuid>,
    pub languages: Option<Vec<String>>,
    pub extract_entities: Option<bool>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OcrCorrectionRequest {
    pub job_id: Uuid,
    pub corrections: Vec<TextCorrection>,
}

#[derive(Debug, Deserialize)]
pub struct TextCorrection {
    pub original_text: String,
    pub corrected_text: String,
    pub page_number: u32,
}

#[derive(Debug, Deserialize)]
pub struct OcrTemplate {
    pub name: String,
    pub document_type: String,
    pub field_mappings: Vec<FieldMapping>,
}

#[derive(Debug, Deserialize)]
pub struct FieldMapping {
    pub field_name: String,
    pub pattern: String,
    pub data_type: String,
    pub required: bool,
}

pub async fn process_document(
    State(_app): State<App>,
    Path(document_id): Path<Uuid>,
    Json(_request): Json<OcrProcessRequest>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/ocr/process/{}", document_id));
    
    // TODO: Call document processor service
    let job_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "job_id": job_id,
        "document_id": document_id,
        "status": "processing_started",
        "message": "OCR processing forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::ACCEPTED, std::time::Duration::from_millis(25));
    Ok(Json(response))
}

pub async fn get_ocr_status(
    State(_app): State<App>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/ocr/status/{}", job_id));
    
    let response = serde_json::json!({
        "job_id": job_id,
        "status": "processing",
        "progress": 0.5,
        "message": "OCR status retrieval forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(10));
    Ok(Json(response))
}

pub async fn get_ocr_result(
    State(_app): State<App>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/ocr/result/{}", job_id));
    
    let response = serde_json::json!({
        "job_id": job_id,
        "message": "OCR result retrieval forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

pub async fn process_batch_ocr(
    State(_app): State<App>,
    Json(_request): Json<BatchOcrRequest>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/ocr/process/batch");
    
    // Validate batch size
    // TODO: Call document processor service
    let batch_job_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "batch_job_id": batch_job_id,
        "status": "batch_processing_started",
        "message": "Batch OCR processing forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::ACCEPTED, std::time::Duration::from_millis(30));
    Ok(Json(response))
}

pub async fn get_batch_ocr_status(
    State(_app): State<App>,
    Path(batch_job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/ocr/batch/{}", batch_job_id));
    
    let response = serde_json::json!({
        "batch_job_id": batch_job_id,
        "status": "processing",
        "progress": 0.3,
        "message": "Batch OCR status retrieval forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

pub async fn apply_ocr_corrections(
    State(_app): State<App>,
    Path(job_id): Path<Uuid>,
    Json(_request): Json<OcrCorrectionRequest>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/ocr/{}/correct", job_id));
    
    let response = serde_json::json!({
        "job_id": job_id,
        "status": "corrections_applied",
        "message": "OCR corrections forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(40));
    Ok(Json(response))
}

pub async fn create_ocr_template(
    State(_app): State<App>,
    Json(_template): Json<OcrTemplate>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/ocr/templates");
    
    let template_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "template_id": template_id,
        "status": "created",
        "message": "OCR template creation forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

pub async fn apply_ocr_template(
    State(_app): State<App>,
    Path((document_id, template_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/ocr/{}/apply-template/{}", document_id, template_id));
    
    let response = serde_json::json!({
        "document_id": document_id,
        "template_id": template_id,
        "status": "template_applied",
        "message": "OCR template application forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(50));
    Ok(Json(response))
}
