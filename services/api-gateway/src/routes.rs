use axum::{
    routing::{get, post},
    Router,
};

use crate::{app::App, handlers, middleware};

pub fn create_router(app: App) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/register", post(handlers::auth::register))
        .route("/patients/:patient_id/timeline", get(handlers::timeline::get_timeline))
        .route("/patients/:patient_id/events", post(handlers::events::create_event))
        .route("/patients/:patient_id/events/:event_id", get(handlers::events::get_event))
        .route("/patients/:patient_id/events", get(handlers::events::list_events))
        .route("/documents/upload", post(handlers::documents::upload_document))
        .route("/documents/upload/batch", post(handlers::documents::upload_batch))
        .route("/documents/:document_id", get(handlers::documents::get_document))
        .route("/documents/:document_id/preview", get(handlers::documents::get_preview))
        .route("/documents/:document_id/metadata", get(handlers::documents::get_metadata))
        .route("/documents/:document_id/extracted", get(handlers::documents::get_extracted_data))
        .route("/documents/:document_id", axum::routing::delete(handlers::documents::delete_document))
        .route("/documents", get(handlers::documents::list_documents))
        .route("/documents/search", get(handlers::documents::search_documents))
        .route("/documents/:document_id/classify", post(handlers::documents::classify_document))
        
        // Manual data input
        .route("/input/manual", post(handlers::manual_input::create_manual_entry))
        .route("/input/symptom", post(handlers::manual_input::create_symptom))
        .route("/input/medication", post(handlers::manual_input::create_medication))
        .route("/input/lab-result", post(handlers::manual_input::create_lab_result))
        .route("/input/doctor-visit", post(handlers::manual_input::create_doctor_visit))
        .route("/input/diagnosis", post(handlers::manual_input::create_diagnosis))
        .route("/input/quick-symptom", post(handlers::manual_input::create_quick_symptom))
        .route("/input/quick-medication", post(handlers::manual_input::create_quick_medication))
        
        // DICOM processing
        .route("/dicom/upload", post(handlers::dicom::upload_dicom))
        .route("/dicom/:document_id/metadata", get(handlers::dicom::get_dicom_metadata))
        .route("/dicom/:document_id/image", get(handlers::dicom::get_dicom_image))
        .route("/dicom/:document_id/studies", get(handlers::dicom::get_dicom_studies))
        .route("/dicom/:document_id/annotations", get(handlers::dicom::get_dicom_annotations))
        .route("/dicom/:document_id/annotations", post(handlers::dicom::add_dicom_annotation))
        
        // OCR processing
        .route("/ocr/process/:document_id", post(handlers::ocr::process_document))
        .route("/ocr/status/:job_id", get(handlers::ocr::get_ocr_status))
        .route("/ocr/result/:job_id", get(handlers::ocr::get_ocr_result))
        .route("/ocr/process/batch", post(handlers::ocr::process_batch_ocr))
        .route("/ocr/batch/:batch_job_id", get(handlers::ocr::get_batch_ocr_status))
        .route("/ocr/:job_id/correct", post(handlers::ocr::apply_ocr_corrections))
        .route("/ocr/templates", post(handlers::ocr::create_ocr_template))
        .route("/ocr/:document_id/apply-template/:template_id", post(handlers::ocr::apply_ocr_template))
        
        // AI reports
        .route("/ai/reports", post(handlers::ai::generate_report))
        
        // Doctor access
        .route("/doctor-access/:patient_id", post(handlers::doctor_access::generate_access))
        .route("/doctor-view/:token", get(handlers::doctor_access::view_report))
        .layer(axum::middleware::from_fn_with_state(app.clone(), middleware::auth_middleware))
        .with_state(app)
}
