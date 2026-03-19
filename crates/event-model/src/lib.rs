use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    SymptomCreated,
    MedicationStarted,
    MedicationStopped,
    LabResultReceived,
    DoctorVisit,
    Diagnosis,
    DocumentUploaded,
    ReminderTriggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalEvent {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub source: String,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl MedicalEvent {
    pub fn new(
        patient_id: Uuid,
        event_type: EventType,
        payload: serde_json::Value,
        source: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_id,
            event_type,
            timestamp: now,
            payload,
            metadata: EventMetadata {
                source,
                version: 1,
                created_at: now,
                updated_at: None,
            },
        }
    }

    pub fn with_timestamp(
        patient_id: Uuid,
        event_type: EventType,
        payload: serde_json::Value,
        source: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_id,
            event_type,
            timestamp,
            payload,
            metadata: EventMetadata {
                source,
                version: 1,
                created_at: now,
                updated_at: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymptomPayload {
    pub name: String,
    pub severity: u8, // 1-10 scale
    pub description: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationPayload {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub prescribed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabResultPayload {
    pub test_name: String,
    pub value: String,
    pub unit: String,
    pub reference_range: Option<String>,
    pub status: String, // normal, high, low, critical
    pub facility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorVisitPayload {
    pub doctor_name: String,
    pub specialty: String,
    pub facility: String,
    pub reason: String,
    pub notes: Option<String>,
    pub follow_up_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPayload {
    pub filename: String,
    pub file_type: String,
    pub file_size: u64,
    pub storage_path: String,
    pub document_type: String, // prescription, lab_report, radiology, etc.
    pub facility: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisPayload {
    pub condition: String,
    pub icd10_code: Option<String>,
    pub diagnosed_by: String,
    pub severity: Option<String>,
    pub acute: bool,
    pub notes: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Invalid event payload: {0}")]
    InvalidPayload(String),
    
    #[error("Unsupported event type")]
    UnsupportedEventType,
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, EventError>;
