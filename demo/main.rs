use std::collections::HashMap;
use std::net::SocketAddr;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Demo application state
#[derive(Clone)]
struct AppState {
    // In-memory storage for demo
    documents: HashMap<Uuid, Document>,
    events: HashMap<Uuid, Event>,
}

#[derive(Debug, Clone, Serialize)]
struct Document {
    id: Uuid,
    patient_id: Uuid,
    filename: String,
    document_type: String,
    ocr_text: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
struct Event {
    id: Uuid,
    patient_id: Uuid,
    event_type: String,
    data: Value,
    created_at: DateTime<Utc>,
}

// Request structures
#[derive(Debug, Deserialize)]
struct SymptomInput {
    patient_id: Uuid,
    name: String,
    severity: u8,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MedicationInput {
    patient_id: Uuid,
    name: String,
    dosage: String,
    frequency: String,
}

#[derive(Debug, Deserialize)]
struct DocumentQuery {
    patient_id: Option<Uuid>,
    limit: Option<usize>,
}

// Health check
async fn health() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "health-os-demo",
        "timestamp": Utc::now(),
        "features": {
            "document_processing": true,
            "manual_input": true,
            "ocr_simulation": true,
            "dicom_support": true,
            "timeline_engine": true
        }
    }))
}

// Document endpoints
async fn upload_document(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let document_id = Uuid::new_v4();
    let patient_id = payload["patient_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    let document = Document {
        id: document_id,
        patient_id,
        filename: payload["filename"].as_str().unwrap_or("document.pdf").to_string(),
        document_type: payload["document_type"].as_str().unwrap_or("medical_report").to_string(),
        ocr_text: Some("Извлеченный текст из документа:\n\nАнализ крови\nГемоглобин: 145 г/л\nЭритроциты: 4.5 млн/мкл\nЛейкоциты: 6.8 тыс/мкл\n\nЗаключение: Показатели в норме".to_string()),
        created_at: Utc::now(),
    };

    // Store in memory (in real app, would store in database)
    let mut documents = state.documents.clone();
    documents.insert(document_id, document);

    tracing::info!("Document uploaded: {}", document_id);

    Ok(Json(serde_json::json!({
        "document_id": document_id,
        "status": "uploaded",
        "ocr_processed": true,
        "extracted_entities": [
            {
                "type": "lab_value",
                "text": "Гемоглобин: 145 г/л",
                "confidence": 0.95
            },
            {
                "type": "lab_value", 
                "text": "Эритроциты: 4.5 млн/мкл",
                "confidence": 0.92
            }
        ],
        "message": "Document processed successfully with OCR"
    })))
}

async fn list_documents(
    State(state): State<AppState>,
    Query(params): Query<DocumentQuery>,
) -> Json<Value> {
    let documents_vec: Vec<Document> = state
        .documents
        .values()
        .filter(|doc| {
            if let Some(patient_id) = params.patient_id {
                doc.patient_id == patient_id
            } else {
                true
            }
        })
        .cloned()
        .collect();

    let limit = params.limit.unwrap_or(documents_vec.len());

    Json(serde_json::json!({
        "documents": documents_vec.clone().into_iter().take(limit).collect::<Vec<_>>(),
        "total": documents_vec.len(),
        "limit": limit
    }))
}

// Manual input endpoints
async fn add_symptom(
    State(state): State<AppState>,
    Json(payload): Json<SymptomInput>,
) -> Result<Json<Value>, StatusCode> {
    if payload.severity < 1 || payload.severity > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let event_id = Uuid::new_v4();
    let event = Event {
        id: event_id,
        patient_id: payload.patient_id,
        event_type: "symptom".to_string(),
        data: serde_json::json!({
            "name": payload.name,
            "severity": payload.severity,
            "description": payload.description
        }),
        created_at: Utc::now(),
    };

    let mut events = state.events.clone();
    events.insert(event_id, event);

    tracing::info!("Symptom added: {} for patient: {}", payload.name, payload.patient_id);

    Ok(Json(serde_json::json!({
        "event_id": event_id,
        "type": "symptom",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Symptom recorded successfully"
    })))
}

async fn add_medication(
    State(state): State<AppState>,
    Json(payload): Json<MedicationInput>,
) -> Result<Json<Value>, StatusCode> {
    let event_id = Uuid::new_v4();
    let event = Event {
        id: event_id,
        patient_id: payload.patient_id,
        event_type: "medication".to_string(),
        data: serde_json::json!({
            "name": payload.name,
            "dosage": payload.dosage,
            "frequency": payload.frequency
        }),
        created_at: Utc::now(),
    };

    let mut events = state.events.clone();
    events.insert(event_id, event);

    tracing::info!("Medication added: {} for patient: {}", payload.name, payload.patient_id);

    Ok(Json(serde_json::json!({
        "event_id": event_id,
        "type": "medication",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Medication recorded successfully"
    })))
}

// Timeline endpoint
async fn get_timeline(
    State(state): State<AppState>,
    Path(patient_id): Path<Uuid>,
) -> Json<Value> {
    let events: Vec<Event> = state
        .events
        .values()
        .filter(|event| event.patient_id == patient_id)
        .cloned()
        .collect();

    let mut sorted_events = events;
    sorted_events.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    Json(serde_json::json!({
        "patient_id": patient_id,
        "events": sorted_events,
        "total_events": sorted_events.len(),
        "generated_at": Utc::now(),
        "summary": {
            "symptoms_count": sorted_events.iter().filter(|e| e.event_type == "symptom").count(),
            "medications_count": sorted_events.iter().filter(|e| e.event_type == "medication").count(),
            "documents_count": state.documents.values().filter(|d| d.patient_id == patient_id).count()
        }
    }))
}

// DICOM endpoint (mock)
async fn upload_dicom(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let document_id = Uuid::new_v4();
    
    tracing::info!("DICOM uploaded: {}", document_id);

    Ok(Json(serde_json::json!({
        "document_id": document_id,
        "modality": payload.get("modality").unwrap_or(&Value::String("CT".to_string())),
        "patient_name": payload.get("patient_name").unwrap_or(&Value::String("Иванов Иван".to_string())),
        "study_date": payload.get("study_date").unwrap_or(&Value::String("20240115".to_string())),
        "status": "uploaded",
        "metadata": {
            "rows": 512,
            "columns": 512,
            "bits_stored": 16,
            "photometric_interpretation": "MONOCHROME2"
        },
        "thumbnail_url": format!("/preview/{}", document_id),
        "message": "DICOM file processed successfully"
    })))
}

// OCR endpoint (mock)
async fn process_ocr(
    State(_state): State<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let job_id = Uuid::new_v4();
    
    tracing::info!("OCR processing started for document: {} (job: {})", document_id, job_id);

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "document_id": document_id,
        "status": "processing_started",
        "estimated_completion": Utc::now() + chrono::Duration::seconds(30),
        "message": "OCR processing started"
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Initialize application state
    let state = AppState {
        documents: HashMap::new(),
        events: HashMap::new(),
    };

    // Create router
    let app = Router::new()
        .route("/health", get(health))
        
        // Document processing
        .route("/documents/upload", post(upload_document))
        .route("/documents", get(list_documents))
        
        // Manual input
        .route("/input/symptom", post(add_symptom))
        .route("/input/medication", post(add_medication))
        
        // Timeline
        .route("/patients/:patient_id/timeline", get(get_timeline))
        
        // DICOM
        .route("/dicom/upload", post(upload_dicom))
        
        // OCR
        .route("/ocr/process/:document_id", post(process_ocr))
        
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("🚀 Health OS Demo Server starting on {}", addr);
    tracing::info!("📊 Available endpoints:");
    tracing::info!("   Health: GET http://localhost:8080/health");
    tracing::info!("   Upload Document: POST http://localhost:8080/documents/upload");
    tracing::info!("   List Documents: GET http://localhost:8080/documents");
    tracing::info!("   Add Symptom: POST http://localhost:8080/input/symptom");
    tracing::info!("   Add Medication: POST http://localhost:8080/input/medication");
    tracing::info!("   Get Timeline: GET http://localhost:8080/patients/{{patient_id}}/timeline");
    tracing::info!("   Upload DICOM: POST http://localhost:8080/dicom/upload");
    tracing::info!("   Process OCR: POST http://localhost:8080/ocr/process/{{document_id}}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
