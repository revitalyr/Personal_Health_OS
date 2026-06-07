use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::app::App;
use telemetry::{trace_request, trace_response};

#[derive(Debug, Deserialize)]
pub struct DocumentQuery {
    pub patient_id: Option<Uuid>,
    pub document_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub patient_id: Option<Uuid>,
    pub document_type: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub file_size: u64,
    pub document_type: Option<String>,
    pub facility: Option<String>,
    pub document_date: Option<chrono::NaiveDate>,
    pub processed: bool,
    pub ocr_extracted_text: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn upload_document(
    State(app): State<App>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/documents/upload");
    
    let mut uploaded_files = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("file").to_string();
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        
        tracing::info!(
            "Processing upload: {} (name: {}, type: {})",
            filename,
            name,
            content_type
        );
        
        // Get file data
        let file_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Validate file size
        if file_data.len() > 50 * 1024 * 1024 { // 50MB limit
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
        
        // Determine document type from filename/content
        let document_type = classify_document_type(&filename, &content_type);
        
        // Store file
        let storage_path = app.document_storage.store_file(&filename, &file_data).await
            .map_err(|e| {
                tracing::error!("Failed to store file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        // Create document record
        let document_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4(); // TODO: Get from auth context
        
        // Store in database
        if let Err(e) = app.document_service.create_document_record(
            document_id,
            patient_id,
            filename.clone(),
            content_type.clone(),
            file_data.len() as u64,
            storage_path,
            document_type.clone(),
        ).await {
            tracing::error!("Failed to create document record: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        
        // Start OCR processing if applicable
        if should_process_ocr(&document_type) {
            if let Err(e) = app.document_service.start_ocr_processing(document_id).await {
                tracing::warn!("Failed to start OCR processing: {}", e);
            }
        }
        
        uploaded_files.push(serde_json::json!({
            "id": document_id,
            "filename": filename,
            "file_type": content_type,
            "document_type": document_type,
            "size": file_data.len(),
            "status": "uploaded",
            "ocr_processing": should_process_ocr(&document_type)
        }));
    }
    
    let response = serde_json::json!({
        "uploaded_files": uploaded_files,
        "total_files": uploaded_files.len(),
        "message": "Documents uploaded successfully"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(100));
    Ok(Json(response))
}

pub async fn upload_batch(
    State(app): State<App>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/documents/upload/batch");
    
    let mut batch_results = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let _content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        
        match field.bytes().await {
            Ok(file_data) => {
                // Process file similar to upload_document
                let document_id = Uuid::new_v4();
                let _storage_path = format!("/batch/{}", document_id);
                
                // Store file (simplified)
                if let Ok(_) = app.document_storage.store_file(&filename, &file_data).await {
                    success_count += 1;
                    batch_results.push(serde_json::json!({
                        "filename": filename,
                        "status": "success",
                        "document_id": document_id
                    }));
                } else {
                    error_count += 1;
                    batch_results.push(serde_json::json!({
                        "filename": filename,
                        "status": "error",
                        "error": "Storage failed"
                    }));
                }
            }
            Err(e) => {
                error_count += 1;
                batch_results.push(serde_json::json!({
                    "filename": filename,
                    "status": "error",
                    "error": format!("Failed to read file: {}", e)
                }));
            }
        }
    }
    
    let response = serde_json::json!({
        "batch_results": batch_results,
        "summary": {
            "total_files": success_count + error_count,
            "successful": success_count,
            "failed": error_count
        },
        "message": "Batch upload completed"
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(200));
    Ok(Json(response))
}

pub async fn get_document(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/documents/{}", document_id));
    
    // Get document from database
    let document = app.document_service.get_document(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to get document: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match document {
        Some(doc) => {
            let response = serde_json::json!({
                "id": doc.id,
                "patient_id": doc.patient_id,
                "filename": doc.filename,
                "file_type": doc.file_type,
                "file_size": doc.file_size,
                "document_type": doc.document_type,
                "facility": doc.facility,
                "document_date": doc.document_date,
                "processed": doc.processed,
                "ocr_extracted_text": doc.ocr_extracted_text,
                "extracted_data": doc.extracted_data,
                "created_at": doc.created_at,
                "updated_at": doc.updated_at
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

pub async fn get_preview(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/documents/{}/preview", document_id));
    
    // Generate preview (thumbnail, first page, etc.)
    let preview_url = app.document_service.generate_preview(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to generate preview: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "preview_url": preview_url,
        "generated_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(50));
    Ok(Json(response))
}

pub async fn get_metadata(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/documents/{}/metadata", document_id));
    
    let metadata = app.document_service.get_document_metadata(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to get metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "metadata": metadata,
        "extracted_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

pub async fn get_extracted_data(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/documents/{}/extracted", document_id));
    
    let extracted_data = app.document_service.get_extracted_data(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to get extracted data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "extracted_data": extracted_data,
        "extracted_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

pub async fn delete_document(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("DELETE", format!("/documents/{}", document_id));
    
    // Delete document from storage and database
    app.document_service.delete_document(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to delete document: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "status": "deleted",
        "deleted_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(25));
    Ok(Json(response))
}

pub async fn list_documents(
    State(app): State<App>,
    Query(params): Query<DocumentQuery>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", "/documents");
    
    let documents = app.document_service.list_documents(
        params.patient_id,
        params.document_type,
        params.limit.unwrap_or(20),
        params.offset.unwrap_or(0),
    ).await
        .map_err(|e| {
            tracing::error!("Failed to list documents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "documents": documents,
        "total_count": documents.len(),
        "limit": params.limit.unwrap_or(20),
        "offset": params.offset.unwrap_or(0)
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(30));
    Ok(Json(response))
}

pub async fn search_documents(
    State(app): State<App>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/documents/search?q={}", params.q));
    
    let results = app.document_service.search_documents(
        &params.q,
        params.patient_id,
        params.document_type,
    ).await
        .map_err(|e| {
            tracing::error!("Failed to search documents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "query": params.q,
        "results": results,
        "total_count": results.len(),
        "searched_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(40));
    Ok(Json(response))
}

pub async fn classify_document(
    State(app): State<App>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/documents/{}/classify", document_id));
    
    let classification = app.document_service.classify_document(document_id).await
        .map_err(|e| {
            tracing::error!("Failed to classify document: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "document_id": document_id,
        "classification": classification,
        "classified_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(60));
    Ok(Json(response))
}

// Helper functions
fn classify_document_type(filename: &str, content_type: &str) -> Option<String> {
    let filename_lower = filename.to_lowercase();
    let content_type_lower = content_type.to_lowercase();
    
    match () {
        _ if filename_lower.contains("recipe") || filename_lower.contains("prescription") => 
            Some("prescription".to_string()),
        _ if filename_lower.contains("analysis") || filename_lower.contains("lab") => 
            Some("lab_report".to_string()),
        _ if filename_lower.contains("x-ray") || filename_lower.contains("mri") || filename_lower.contains("ct") => 
            Some("radiology".to_string()),
        _ if filename_lower.contains("discharge") || filename_lower.contains("выписка") => 
            Some("discharge_summary".to_string()),
        _ if content_type_lower.contains("pdf") => 
            Some("medical_document".to_string()),
        _ if content_type_lower.contains("image") => 
            Some("medical_image".to_string()),
        _ => None,
    }
}

fn should_process_ocr(document_type: &Option<String>) -> bool {
    match document_type.as_deref() {
        Some("prescription") | Some("lab_report") | Some("discharge_summary") | 
        Some("medical_document") | Some("other") => true,
        Some("radiology") => false, // DICOM images have their own metadata
        _ => true, // Process OCR for unknown types
    }
}
