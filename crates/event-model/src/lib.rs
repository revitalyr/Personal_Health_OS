use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of medical events in the system.
///
/// Each variant maps to a specific domain operation:
/// - `SymptomCreated`: patient-reported symptom
/// - `MedicationStarted`/`MedicationStopped`: medication changes
/// - `LabResultReceived`: lab test results
/// - `DoctorVisit`: medical consultation
/// - `Diagnosis`: clinical diagnosis
/// - `DocumentUploaded`: document attached to patient record
/// - `ReminderTriggered`: automated notification event
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

/// A single medical event in the append-only event log.
///
/// Events are immutable after creation. Each event carries a typed payload
/// in the `payload` JSONB field and metadata about origin and version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalEvent {
    /// Unique event identifier
    pub id: Uuid,
    /// Patient this event belongs to
    pub patient_id: Uuid,
    /// Domain event type
    pub event_type: EventType,
    /// When the event occurred (may differ from created_at)
    pub timestamp: DateTime<Utc>,
    /// Typed payload as serialized JSON (SymptomPayload, MedicationPayload, etc.)
    pub payload: serde_json::Value,
    /// Origin and versioning metadata
    pub metadata: EventMetadata,
}

/// Metadata about the origin and version of a MedicalEvent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Source system or service that created the event
    pub source: String,
    /// Event schema version (for forward compatibility)
    pub version: u32,
    /// When this event record was created
    pub created_at: DateTime<Utc>,
    /// When this event record was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

impl MedicalEvent {
    /// Create a new MedicalEvent with the current timestamp.
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

    /// Create a new MedicalEvent with a specific timestamp (for backfilling).
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

/// Payload for a patient-reported symptom event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymptomPayload {
    pub name: String,
    pub severity: u8, // 1-10 scale
    pub description: Option<String>,
    pub duration: Option<String>,
}

/// Payload for a medication start/stop event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationPayload {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub prescribed_by: Option<String>,
}

/// Payload for a lab test result event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabResultPayload {
    pub test_name: String,
    pub value: String,
    pub unit: String,
    pub reference_range: Option<String>,
    pub status: String, // normal, high, low, critical
    pub facility: String,
}

/// Payload for a doctor visit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorVisitPayload {
    pub doctor_name: String,
    pub specialty: String,
    pub facility: String,
    pub reason: String,
    pub notes: Option<String>,
    pub follow_up_date: Option<DateTime<Utc>>,
}

/// Payload for a document upload event.
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

/// Payload for a diagnosis event (with optional ICD-10 code).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisPayload {
    pub condition: String,
    pub icd10_code: Option<String>,
    pub diagnosed_by: String,
    pub severity: Option<String>,
    pub acute: bool,
    pub notes: Option<String>,
}

/// Errors that can occur during event construction or serialization.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Invalid event payload: {0}")]
    InvalidPayload(String),
    
    #[error("Unsupported event type")]
    UnsupportedEventType,
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Convenience alias for results using EventError.
pub type Result<T> = std::result::Result<T, EventError>;
