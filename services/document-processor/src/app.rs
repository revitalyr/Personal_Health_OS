use storage::{EventStore, DatabaseConfig};
use std::sync::Arc;
use crate::{services::DocumentService, nats::NatsClient, storage::DocumentStorage};

#[derive(Clone)]
pub struct App {
    pub event_store: Arc<EventStore>,
    pub document_service: Arc<DocumentService>,
    pub nats_client: Arc<NatsClient>,
    pub document_storage: Arc<DocumentStorage>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        // Initialize database connection
        let db_config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "health_os".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            max_connections: 10,
        };

        let event_store = Arc::new(EventStore::new(db_config).await?);

        // Initialize NATS client
        let nats_client = Arc::new(NatsClient::new(&config.nats_url).await?);

        // Initialize document storage
        let document_storage = Arc::new(DocumentStorage::new(config.clone()).await?);

        // Initialize document service
        let document_service = Arc::new(DocumentService::new(
            event_store.clone(),
            nats_client.clone(),
            document_storage.clone(),
            config.clone(),
        ));

        tracing::info!("Document Processor application initialized");

        Ok(Self {
            event_store,
            document_service,
            nats_client,
            document_storage,
        })
    }
}
