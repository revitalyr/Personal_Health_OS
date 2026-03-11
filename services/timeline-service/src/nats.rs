use async_nats::Client;
use event_model::MedicalEvent;
use serde_json;
use thiserror::Error;
use tracing::{info, warn, error};

#[derive(Debug, Error)]
pub enum NatsError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Publish error: {0}")]
    Publish(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct NatsClient {
    client: Client,
}

impl NatsClient {
    pub async fn new(nats_url: &str) -> Result<Self, NatsError> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| NatsError::Connection(e.to_string()))?;

        info!("Connected to NATS at {}", nats_url);
        
        Ok(Self { client })
    }

    pub async fn publish_event(&self, event: &MedicalEvent) -> Result<(), NatsError> {
        let subject = format!("medical_events.{}", event.patient_id);
        let payload = serde_json::to_vec(event)?;
        
        self.client
            .publish(subject, payload.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published event {} to NATS", event.id);
        Ok(())
    }

    pub async fn publish_timeline_update(&self, patient_id: uuid::Uuid) -> Result<(), NatsError> {
        let subject = format!("timeline_updated.{}", patient_id);
        let payload = serde_json::json!({
            "patient_id": patient_id,
            "timestamp": chrono::Utc::now(),
            "event_type": "timeline_updated"
        });
        
        let payload_bytes = serde_json::to_vec(&payload)?;
        
        self.client
            .publish(subject, payload_bytes.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        info!("Published timeline update for patient {}", patient_id);
        Ok(())
    }

    pub async fn subscribe_to_events<F>(&self, patient_id: uuid::Uuid, callback: F) -> Result<(), NatsError>
    where
        F: Fn(MedicalEvent) + Send + Sync + 'static,
    {
        let subject = format!("medical_events.{}", patient_id);
        let mut subscriber = self.client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::Connection(e.to_string()))?;

        info!("Subscribed to events for patient {}", patient_id);

        // Spawn a task to handle incoming messages
        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<MedicalEvent>(&message.payload) {
                    Ok(event) => {
                        info!("Received event {} for patient {}", event.id, event.patient_id);
                        callback(event);
                    }
                    Err(e) => {
                        error!("Failed to deserialize event: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
