use storage::EventStore;
use event_model::MedicalEvent;
use std::sync::Arc;
use crate::nats::NatsClient;

/// Coordinates timeline operations by bridging an event store with a NATS
/// messaging layer for publishing and querying medical timeline events.
pub struct TimelineService {
    event_store: Arc<dyn EventStore>,
    nats_client: Arc<NatsClient>,
}

impl TimelineService {
    /// Creates a new `TimelineService` with the given event store and NATS client.
    pub fn new(event_store: Arc<dyn EventStore>, nats_client: Arc<NatsClient>) -> Self {
        Self {
            event_store,
            nats_client,
        }
    }

    /// Persists a medical event to the event store and publishes it via NATS.
    pub async fn create_event(&self, event: &MedicalEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.event_store.store_event(event).await?;

        if let Err(e) = self.nats_client.publish_event(event).await {
            tracing::warn!("Failed to publish event to NATS: {}", e);
        }

        tracing::info!("Created and published event: {} for patient: {}", event.id, event.patient_id);
        Ok(())
    }

    /// Retrieves all events for a given patient, ordered by timestamp.
    pub async fn get_timeline(&self, patient_id: uuid::Uuid) -> Result<Vec<MedicalEvent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.event_store.get_events_by_patient(patient_id).await?)
    }

    /// Fetches a single medical event by its unique identifier.
    pub async fn get_event(&self, event_id: uuid::Uuid) -> Result<Option<MedicalEvent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.event_store.get_event_by_id(event_id).await?)
    }
}
