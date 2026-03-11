use storage::{EventStore, Result as StorageResult};
use event_model::MedicalEvent;
use std::sync::Arc;
use crate::nats::NatsClient;

pub struct TimelineService {
    event_store: Arc<EventStore>,
    nats_client: Arc<NatsClient>,
}

impl TimelineService {
    pub fn new(event_store: Arc<EventStore>, nats_client: Arc<NatsClient>) -> Self {
        Self {
            event_store,
            nats_client,
        }
    }

    pub async fn create_event(&self, event: &MedicalEvent) -> StorageResult<()> {
        // Store event in database
        self.event_store.store_event(event).await?;

        // Publish event to NATS for other services
        if let Err(e) = self.nats_client.publish_event(event).await {
            tracing::warn!("Failed to publish event to NATS: {}", e);
        }

        tracing::info!("Created and published event: {} for patient: {}", event.id, event.patient_id);
        Ok(())
    }

    pub async fn get_timeline(&self, patient_id: uuid::Uuid) -> StorageResult<Vec<MedicalEvent>> {
        self.event_store.get_events_by_patient(patient_id).await
    }

    pub async fn get_event(&self, event_id: uuid::Uuid) -> StorageResult<Option<MedicalEvent>> {
        self.event_store.get_event_by_id(event_id).await
    }
}
