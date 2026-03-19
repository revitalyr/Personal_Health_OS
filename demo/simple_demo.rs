use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use uuid::Uuid;
use chrono::Utc;

// Simple demo without complex state management
#[derive(Debug, Clone, Serialize)]
struct Document {
    id: Uuid,
    patient_id: Uuid,
    filename: String,
    document_type: String,
    ocr_text: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
struct Event {
    id: Uuid,
    patient_id: Uuid,
    event_type: String,
    data: Value,
    created_at: chrono::DateTime<Utc>,
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
    Query(params): Query<DocumentQuery>,
) -> Json<Value> {
    let documents = vec![
        Document {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            filename: "blood_test.pdf".to_string(),
            document_type: "lab_report".to_string(),
            ocr_text: Some("Анализ крови".to_string()),
            created_at: Utc::now(),
        }
    ];

    let limit = params.limit.unwrap_or(documents.len());

    Json(serde_json::json!({
        "documents": documents.into_iter().take(limit).collect::<Vec<_>>(),
        "total": documents.len(),
        "limit": limit
    }))
}

// Manual input endpoints
async fn add_symptom(
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
    Path(patient_id): Path<Uuid>,
) -> Json<Value> {
    let events = vec![
        Event {
            id: Uuid::new_v4(),
            patient_id,
            event_type: "symptom".to_string(),
            data: serde_json::json!({
                "name": "Головная боль",
                "severity": 6,
                "description": "Пульсирующая боль в лобной части"
            }),
            created_at: Utc::now(),
        },
        Event {
            id: Uuid::new_v4(),
            patient_id,
            event_type: "medication".to_string(),
            data: serde_json::json!({
                "name": "Ибупрофен",
                "dosage": "400мг",
                "frequency": "3 раза в день"
            }),
            created_at: Utc::now(),
        }
    ];

    Json(serde_json::json!({
        "patient_id": patient_id,
        "events": events,
        "total_events": events.len(),
        "generated_at": Utc::now(),
        "summary": {
            "symptoms_count": 1,
            "medications_count": 1,
            "documents_count": 1
        }
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create router
    let app = Router::new()
        .route("/health", get(health))
        .route("/documents/upload", post(upload_document))
        .route("/documents", get(list_documents))
        .route("/input/symptom", post(add_symptom))
        .route("/input/medication", post(add_medication))
        .route("/patients/:patient_id/timeline", get(get_timeline));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("🚀 Health OS Simple Demo Server starting on {}", addr);
    tracing::info!("📊 Available endpoints:");
    tracing::info!("   Health: GET http://localhost:8080/health");
    tracing::info!("   Upload Document: POST http://localhost:8080/documents/upload");
    tracing::info!("   List Documents: GET http://localhost:8080/documents");
    tracing::info!("   Add Symptom: POST http://localhost:8080/input/symptom");
    tracing::info!("   Add Medication: POST http://localhost:8080/input/medication");
    tracing::info!("   Get Timeline: GET http://localhost:8080/patients/{{patient_id}}/timeline");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
