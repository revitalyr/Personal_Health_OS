use chrono::{DateTime, Utc};
use event_model::{MedicalEvent, EventType, Result, EventError};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

const MAX_EVENTS_PER_DAY: usize = 50;

/// Timeline representation for a patient's medical events
#[derive(Debug, Clone, Serialize)]
pub struct Timeline {
    /// Patient ID
    pub patient_id: Uuid,
    /// Ordered list of medical events
    pub events: Vec<MedicalEvent>,
    /// Timestamp when this timeline was generated
    pub generated_at: DateTime<Utc>,
}

/// Summary statistics for a timeline
#[derive(Debug, Clone)]
pub struct TimelineSummary {
    /// Total number of events
    pub total_events: usize,
    /// Date range (start, end)
    pub date_range: (DateTime<Utc>, DateTime<Utc>),
    /// Count of events by type
    pub event_types: HashMap<EventType, usize>,
    /// Most recent events (last 10)
    pub recent_events: Vec<MedicalEvent>,
}

/// Filter options for timeline queries
#[derive(Debug, Clone)]
pub struct TimelineFilter {
    /// Optional start date for filtering
    pub start_date: Option<DateTime<Utc>>,
    /// Optional end date for filtering
    pub end_date: Option<DateTime<Utc>>,
    /// Optional event type filter
    pub event_types: Option<Vec<EventType>>,
    /// Optional limit on number of events
    pub limit: Option<usize>,
}

impl Default for TimelineFilter {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            event_types: None,
            limit: None,
        }
    }
}

/// Timeline engine for building and analyzing medical event timelines
pub struct TimelineEngine;

impl TimelineEngine {
    /// Build a timeline from a list of medical events
    /// 
    /// Events are sorted by timestamp in ascending order
    pub fn build_timeline(events: Vec<MedicalEvent>) -> Timeline {
        let patient_id = events.first().map(|e| e.patient_id).unwrap_or_default();
        let mut sorted_events = events;
        
        // Sort events by timestamp
        sorted_events.sort_by_key(|e| e.timestamp);
        
        Timeline {
            patient_id,
            events: sorted_events,
            generated_at: Utc::now(),
        }
    }

    /// Build a filtered timeline from events based on filter criteria
    pub fn build_filtered_timeline(
        events: Vec<MedicalEvent>,
        filter: TimelineFilter,
    ) -> Result<Timeline> {
        let patient_id = events.first().map(|e| e.patient_id).unwrap_or_default();
        
        let mut filtered_events: Vec<MedicalEvent> = events
            .into_iter()
            .filter(|event| {
                // Date range filter
                if let Some(start) = filter.start_date {
                    if event.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = filter.end_date {
                    if event.timestamp > end {
                        return false;
                    }
                }
                
                // Event type filter
                if let Some(ref types) = filter.event_types {
                    if !types.contains(&event.event_type) {
                        return false;
                    }
                }
                
                true
            })
            .collect();
        
        // Sort by timestamp
        filtered_events.sort_by_key(|e| e.timestamp);
        
        // Apply limit
        if let Some(limit) = filter.limit {
            filtered_events.truncate(limit);
        }
        
        Ok(Timeline {
            patient_id,
            events: filtered_events,
            generated_at: Utc::now(),
        })
    }

    /// Generate a summary statistics for a timeline
    pub fn generate_summary(timeline: &Timeline) -> TimelineSummary {
        let total_events = timeline.events.len();
        
        let (start_date, end_date) = if timeline.events.is_empty() {
            (Utc::now(), Utc::now())
        } else {
            let first = timeline.events.first().unwrap().timestamp;
            let last = timeline.events.last().unwrap().timestamp;
            (first, last)
        };
        
        let mut event_types: HashMap<EventType, usize> = HashMap::new();
        for event in &timeline.events {
            *event_types.entry(event.event_type.clone()).or_insert(0) += 1;
        }
        
        let recent_events = timeline.events
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect();
        
        TimelineSummary {
            total_events,
            date_range: (start_date, end_date),
            event_types,
            recent_events,
        }
    }

    pub fn find_related_events(timeline: &Timeline, event_id: Uuid) -> Vec<&MedicalEvent> {
        let target_event = timeline.events.iter().find(|e| e.id == event_id);
        
        match target_event {
            Some(target) => {
                let target_date = target.timestamp.date_naive();
                
                timeline.events
                    .iter()
                    .filter(|e| {
                        e.id != event_id && 
                        (e.timestamp.date_naive() - target_date).num_days().abs() <= 7
                    })
                    .collect()
            }
            None => Vec::new(),
        }
    }

    /// Detect anomalies in the timeline (duplicates, future events, unusual patterns)
    pub fn detect_anomalies(timeline: &Timeline) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        // Check for duplicate events
        let mut seen = HashMap::new();
        for event in &timeline.events {
            let key = format!("{:?}_{:?}", event.event_type, event.timestamp);
            if seen.contains_key(&key) {
                anomalies.push(Anomaly::DuplicateEvent {
                    event_id: event.id,
                    timestamp: event.timestamp,
                });
            }
            seen.insert(key, true);
        }
        
        // Check for events in the future
        let now = Utc::now();
        for event in &timeline.events {
            if event.timestamp > now {
                anomalies.push(Anomaly::FutureEvent {
                    event_id: event.id,
                    timestamp: event.timestamp,
                });
            }
        }
        
        // Check for unusual patterns
        anomalies.extend(Self::detect_unusual_patterns(timeline));
        
        anomalies
    }

    fn detect_unusual_patterns(timeline: &Timeline) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        // Group events by day
        let mut events_by_day: HashMap<chrono::NaiveDate, Vec<&MedicalEvent>> = HashMap::new();
        for event in &timeline.events {
            events_by_day
                .entry(event.timestamp.date_naive())
                .or_default()
                .push(event);
        }
        
        // Check for days with too many events (potential data entry error)
        for (date, events) in events_by_day {
            if events.len() > MAX_EVENTS_PER_DAY {
                anomalies.push(Anomaly::HighEventFrequency {
                    date,
                    event_count: events.len(),
                });
            }
        }
        
        anomalies
    }

    /// Export timeline to specified format (JSON, Text, or CSV)
    pub fn export_timeline(timeline: &Timeline, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(timeline)
                    .map_err(|e| EventError::SerializationError(e))
            }
            ExportFormat::Text => {
                Ok(Self::format_as_text(timeline))
            }
            ExportFormat::Csv => {
                Ok(Self::format_as_csv(timeline))
            }
        }
    }

    fn format_as_text(timeline: &Timeline) -> String {
        let mut output = String::new();
        output.push_str(&format!("Medical Timeline for Patient: {}\n", timeline.patient_id));
        output.push_str(&format!("Generated: {}\n\n", timeline.generated_at));
        
        for event in &timeline.events {
            output.push_str(&format!(
                "{} | {:?} | {}\n",
                event.timestamp.format("%Y-%m-%d %H:%M"),
                event.event_type,
                Self::format_event_summary(event)
            ));
        }
        
        output
    }

    fn format_as_csv(timeline: &Timeline) -> String {
        let mut output = String::new();
        output.push_str("Timestamp,Event Type,Summary\n");
        
        for event in &timeline.events {
            output.push_str(&format!(
                "{},{:?},{}\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                event.event_type,
                Self::format_event_summary(event).replace(',', ";")
            ));
        }
        
        output
    }

    fn format_event_summary(event: &MedicalEvent) -> String {
        match event.event_type {
            EventType::SymptomCreated => {
                if let Ok(symptom) = serde_json::from_value::<event_model::SymptomPayload>(event.payload.clone()) {
                    format!("Symptom: {} (Severity: {})", symptom.name, symptom.severity)
                } else {
                    "Symptom recorded".to_string()
                }
            }
            EventType::MedicationStarted => {
                if let Ok(med) = serde_json::from_value::<event_model::MedicationPayload>(event.payload.clone()) {
                    format!("Started: {} {}", med.name, med.dosage)
                } else {
                    "Medication started".to_string()
                }
            }
            EventType::LabResultReceived => {
                if let Ok(lab) = serde_json::from_value::<event_model::LabResultPayload>(event.payload.clone()) {
                    format!("Lab: {} = {} {} ({})", lab.test_name, lab.value, lab.unit, lab.status)
                } else {
                    "Lab result received".to_string()
                }
            }
            _ => format!("{:?}", event.event_type),
        }
    }
}

/// Anomaly detected in a timeline
#[derive(Debug, Clone)]
pub enum Anomaly {
    /// Duplicate event detected (same type and timestamp)
    DuplicateEvent {
        event_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Event with timestamp in the future
    FutureEvent {
        event_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Unusually high event frequency on a specific day
    HighEventFrequency {
        date: chrono::NaiveDate,
        event_count: usize,
    },
}

/// Export format for timeline data
#[derive(Debug, Clone)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// Plain text format
    Text,
    /// CSV format
    Csv,
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_model::{SymptomPayload, EventType};

    #[test]
    fn test_timeline_building() {
        let patient_id = Uuid::new_v4();
        let event1 = MedicalEvent::new(
            patient_id,
            EventType::SymptomCreated,
            serde_json::to_value(SymptomPayload {
                name: "Headache".to_string(),
                severity: 5,
                description: None,
                duration: None,
            }).unwrap(),
            "mobile_app".to_string(),
        );
        
        let event2 = MedicalEvent::new(
            patient_id,
            EventType::SymptomCreated,
            serde_json::to_value(SymptomPayload {
                name: "Fever".to_string(),
                severity: 3,
                description: None,
                duration: None,
            }).unwrap(),
            "mobile_app".to_string(),
        );
        
        let timeline = TimelineEngine::build_timeline(vec![event2, event1]);
        
        assert_eq!(timeline.events.len(), 2);
        assert_eq!(timeline.patient_id, patient_id);
        assert_eq!(timeline.events[0].payload["name"], "Fever");
        assert_eq!(timeline.events[1].payload["name"], "Headache");
    }

    #[test]
    fn test_timeline_filtering() {
        let patient_id = Uuid::new_v4();
        let base_time = Utc::now();
        
        let event1 = MedicalEvent::with_timestamp(
            patient_id,
            EventType::SymptomCreated,
            serde_json::json!({"name": "Headache"}),
            "mobile_app".to_string(),
            base_time - chrono::Duration::days(10),
        );
        
        let event2 = MedicalEvent::with_timestamp(
            patient_id,
            EventType::SymptomCreated,
            serde_json::json!({"name": "Fever"}),
            "mobile_app".to_string(),
            base_time,
        );
        
        let filter = TimelineFilter {
            start_date: Some(base_time - chrono::Duration::days(5)),
            end_date: None,
            event_types: None,
            limit: None,
        };
        
        let timeline = TimelineEngine::build_filtered_timeline(vec![event1, event2], filter).unwrap();
        
        assert_eq!(timeline.events.len(), 1);
        assert_eq!(timeline.events[0].payload["name"], "Fever");
    }
}
