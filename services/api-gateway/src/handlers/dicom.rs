use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize};
use serde_json::Value;
use uuid::Uuid;

use telemetry::{trace_request, trace_response};

use crate::app::App;

/// Query parameters for DICOM image requests.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DicomQuery {
    pub include_metadata: Option<bool>,
    pub include_thumbnails: Option<bool>,
}

/// Request body for POST /dicom/{document_id}/annotations.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DicomAnnotation {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub annotation_type: String,
    pub label: String,
    pub notes: Option<String>,
}

/// POST /dicom/upload — Upload a DICOM file.
pub async fn upload_dicom(
    State(_app): State<App>,
    mut _multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/dicom/upload");
    
    // TODO: Call document processor service
    let response = serde_json::json!({
        "message": "DICOM upload forwarded to processor",
        "service": "document-processor",
        "status": "processing"
    });

    trace_response!(StatusCode::ACCEPTED, std::time::Duration::from_millis(150));
    Ok(Json(response))
}

/// GET /dicom/{document_id}/metadata — Retrieve DICOM metadata.
pub async fn get_dicom_metadata(
    State(_app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/{}/metadata", document_id));
    
    // TODO: Call document processor service
    let response = serde_json::json!({
        "document_id": document_id,
        "message": "DICOM metadata retrieval forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

/// GET /dicom/{document_id}/image — Retrieve a rendered DICOM image.
pub async fn get_dicom_image(
    State(_app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/{}/image", document_id));
    
    let response = serde_json::json!({
        "document_id": document_id,
        "image_url": format!("/dicom-image/{}", document_id),
        "message": "DICOM image retrieval forwarded to processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(100));
    Ok(Json(response))
}

/// GET /dicom/patients/{patient_id}/studies — List DICOM studies for a patient.
pub async fn get_dicom_studies(
    State(_app): State<App>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/patients/{}/studies", patient_id));
    
    let response = serde_json::json!({
        "patient_id": patient_id,
        "message": "DICOM studies retrieval forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(40));
    Ok(Json(response))
}

/// GET /dicom/{document_id}/annotations — Retrieve annotations on a DICOM image.
pub async fn get_dicom_annotations(
    State(_app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/{}/annotations", document_id));
    
    let response = serde_json::json!({
        "document_id": document_id,
        "message": "DICOM annotations retrieval forwarded to processor",
        "service": "document-processor"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

/// POST /dicom/{document_id}/annotations — Add an annotation to a DICOM image.
pub async fn add_dicom_annotation(
    State(_app): State<App>,
    Path(document_id): Path<Uuid>,
    Json(_annotation): Json<DicomAnnotation>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/dicom/{}/annotations", document_id));
    
    let annotation_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "document_id": document_id,
        "annotation_id": annotation_id,
        "status": "created",
        "message": "DICOM annotation creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}
