use axum::{
    routing::{get, post, delete},
    Router,
    middleware as axum_middleware,
};

use crate::{app::App, handlers, middleware as app_middleware};

pub fn create_router(app: App) -> Router {
    Router::new()
        // Health and status
        .route("/health", get(handlers::health))
        
        // Document upload and processing
        .route("/documents/upload", post(handlers::documents::upload_document))
        .route("/documents/upload/batch", post(handlers::documents::upload_batch))
        .route("/documents/:document_id", get(handlers::documents::get_document))
        .route("/documents/:document_id/preview", get(handlers::documents::get_preview))
        .route("/documents/:document_id/metadata", get(handlers::documents::get_metadata))
        .route("/documents/:document_id/extracted", get(handlers::documents::get_extracted_data))
        .route("/documents/:document_id", delete(handlers::documents::delete_document))
        
        // Manual data input
        .route("/input/manual", post(handlers::manual_input::create_manual_entry))
        .route("/input/symptom", post(handlers::manual_input::create_symptom))
        .route("/input/medication", post(handlers::manual_input::create_medication))
        .route("/input/lab-result", post(handlers::manual_input::create_lab_result))
        .route("/input/doctor-visit", post(handlers::manual_input::create_doctor_visit))
        .route("/input/diagnosis", post(handlers::manual_input::create_diagnosis))
        
        // DICOM processing
        .route("/dicom/upload", post(handlers::dicom::upload_dicom))
        .route("/dicom/:document_id/metadata", get(handlers::dicom::get_dicom_metadata))
        .route("/dicom/:document_id/image", get(handlers::dicom::get_dicom_image))
        
        // OCR processing
        .route("/ocr/process/:document_id", post(handlers::ocr::process_document))
        .route("/ocr/status/:job_id", get(handlers::ocr::get_ocr_status))
        .route("/ocr/result/:job_id", get(handlers::ocr::get_ocr_result))
        
        // Document management
        .route("/documents", get(handlers::documents::list_documents))
        .route("/documents/search", get(handlers::documents::search_documents))
        .route("/documents/:document_id/classify", post(handlers::documents::classify_document))
        
        .layer(axum_middleware::from_fn_with_state(app.clone(), app_middleware::auth_middleware))
        .with_state(app)
}
