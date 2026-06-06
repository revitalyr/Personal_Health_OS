use axum::{
    extract::{Path, State, Multipart, Extension},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::app::App;
use telemetry::{trace_request, trace_response};

#[derive(Debug, Deserialize)]
pub struct DicomQuery {
    pub include_metadata: Option<bool>,
    pub include_thumbnails: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DicomMetadata {
    pub patient_id: String,
    pub patient_name: String,
    pub study_date: String,
    pub modality: String,
    pub study_description: String,
    pub series_description: String,
    pub body_part_examined: String,
    pub institution_name: String,
    pub physician_name: String,
    pub image_type: Vec<String>,
    pub rows: u32,
    pub columns: u32,
    pub bits_allocated: u32,
    pub bits_stored: u32,
    pub high_bit: u32,
    pub photometric_interpretation: String,
    pub samples_per_pixel: u32,
    pub planar_configuration: Option<u32>,
    pub pixel_representation: u32,
    pub window_center: Option<Vec<f32>>,
    pub window_width: Option<Vec<f32>>,
    pub rescale_intercept: Option<f32>,
    pub rescale_slope: Option<f32>,
}

pub async fn upload_dicom(
    State(app): State<App>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/dicom/upload");
    
    let mut uploaded_files = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let filename = field.file_name().unwrap_or("unknown.dcm").to_string();
        
        tracing::info!("Processing DICOM upload: {}", filename);
        
        // Get DICOM file data
        let dicom_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Validate DICOM file
        let dicom_metadata = match app.document_service.validate_and_extract_dicom_metadata(&dicom_data).await {
            Ok(metadata) => metadata,
            Err(e) => {
                tracing::error!("Invalid DICOM file {}: {}", filename, e);
                uploaded_files.push(serde_json::json!({
                    "filename": filename,
                    "status": "error",
                    "error": "Invalid DICOM format"
                }));
                continue;
            }
        };
        
        // Store DICOM file
        let storage_path = app.document_storage.store_file(&filename, &dicom_data).await
            .map_err(|e| {
                tracing::error!("Failed to store DICOM file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        // Create document record
        let document_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4(); // TODO: Map DICOM patient ID to system patient ID
        
        if let Err(e) = app.document_service.create_dicom_document_record(
            document_id,
            patient_id,
            filename.clone(),
            storage_path.clone(),
            dicom_metadata.clone(),
        ).await {
            tracing::error!("Failed to create DICOM document record: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        
        // Generate thumbnail if requested
        let thumbnail_url = app.document_service.generate_dicom_thumbnail(document_id).await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to generate DICOM thumbnail: {}", e);
                None
            });
        
        uploaded_files.push(serde_json::json!({
            "document_id": document_id,
            "filename": filename,
            "status": "uploaded",
            "metadata": dicom_metadata,
            "thumbnail_url": thumbnail_url,
            "storage_path": storage_path
        }));
    }
    
    let response = serde_json::json!({
        "uploaded_files": uploaded_files,
        "total_files": uploaded_files.len(),
        "message": "DICOM files uploaded successfully"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(150));
    Ok(Json(response))
}

pub async fn get_dicom_metadata(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/{}/metadata", document_id));
    
    let metadata = app.document_service.get_dicom_metadata(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to get DICOM metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match metadata {
        Some(dicom_metadata) => {
            let response = serde_json::json!({
                "document_id": document_id,
                "dicom_metadata": dicom_metadata,
                "extracted_at": chrono::Utc::now()
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

pub async fn get_dicom_image(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/{}/image", document_id));
    
    // Get DICOM image data (convert to viewable format)
    let image_data = app.document_service.get_dicom_image(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to get DICOM image: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match image_data {
        Some(image_url) => {
            let response = serde_json::json!({
                "document_id": document_id,
                "image_url": image_url,
                "generated_at": chrono::Utc::now()
            });

            trace_response!(StatusCode::OK, std::time::Duration::from_millis(100));
            Ok(Json(response))
        }
        None => {
            trace_response!(StatusCode::NOT_FOUND, std::time::Duration::from_millis(5));
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// DICOM study management
pub async fn get_dicom_studies(
    State(app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/patients/{}/studies", patient_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let studies = app.document_service.get_dicom_studies_for_patient(patient_id).await
        .map_err(|e| {
            tracing::error!("Failed to get DICOM studies: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "patient_id": patient_id,
        "studies": studies,
        "total_studies": studies.len(),
        "retrieved_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(40));
    Ok(Json(response))
}

pub async fn get_dicom_series(
    State(app): State<App>,
    Path((patient_id, study_id)): Path<(Uuid, String)>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/patients/{}/studies/{}/series", patient_id, study_id));
    
    let series = app.document_service.get_dicom_series_for_study(patient_id, &study_id).await
        .map_err(|e| {
            tracing::error!("Failed to get DICOM series: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "patient_id": patient_id,
        "study_id": study_id,
        "series": series,
        "total_series": series.len(),
        "retrieved_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(50));
    Ok(Json(response))
}

// DICOM annotation
#[derive(Debug, Deserialize, Serialize)]
pub struct DicomAnnotation {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub annotation_type: String, // "rectangle", "circle", "point", "line"
    pub label: String,
    pub notes: Option<String>,
    pub created_by: String,
}

pub async fn add_dicom_annotation(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
    Json(annotation): Json<DicomAnnotation>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/dicom/{}/annotations", document_id));
    
    let annotation_value = serde_json::to_value(annotation)
        .map_err(|e| {
            tracing::error!("Failed to serialize annotation: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let annotation_id = app.document_service.add_dicom_annotation(document_id, annotation_value).await
        .map_err(|e| {
            tracing::error!("Failed to add DICOM annotation: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "annotation_id": annotation_id,
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

pub async fn get_dicom_annotations(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/dicom/{}/annotations", document_id));
    
    let annotations = app.document_service.get_dicom_annotations(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to get DICOM annotations: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "annotations": annotations,
        "total_annotations": annotations.len(),
        "retrieved_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
    Ok(Json(response))
}
