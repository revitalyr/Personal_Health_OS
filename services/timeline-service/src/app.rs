use storage::{PostgresEventStore, DatabaseConfig};
use std::sync::Arc;
use crate::{services::TimelineService, nats::NatsClient};
use auth::AuthService;

#[derive(Clone)]
pub struct App {
    /// The underlying event store (Postgres) for persisting timeline events.
    pub event_store: Arc<dyn storage::EventStore>,
    /// Business-logic layer for timeline operations.
    pub timeline_service: Arc<TimelineService>,
    /// NATS client for publishing and subscribing to real-time event notifications.
    pub nats_client: Arc<NatsClient>,
    /// Authentication and authorization service.
    pub auth_service: Arc<AuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        let db_config = DatabaseConfig {
            host: config.database_url.split('@').collect::<Vec<&str>>()[1]
                .split(':').collect::<Vec<&str>>()[0].to_string(),
            port: 5432,
            database: "health_os".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            max_connections: config.max_connections,
        };

        let pg_store = PostgresEventStore::new(db_config).await?;
        let event_store: Arc<dyn storage::EventStore> = Arc::new(pg_store);

        let nats_client = Arc::new(NatsClient::new(&config.nats_url).await?);

        let timeline_service = Arc::new(TimelineService::new(
            event_store.clone(),
            nats_client.clone(),
        ));

        let auth_service = Arc::new(AuthService::new(&config.jwt_secret)?);

        tracing::info!("Timeline Service application initialized");

        Ok(Self {
            event_store,
            timeline_service,
            nats_client,
            auth_service,
        })
    }
}
