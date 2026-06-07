use storage::{EventStore, DatabaseConfig};
use std::sync::Arc;
use crate::{services::TimelineService, nats::NatsClient};
use auth::AuthService;

#[derive(Clone)]
pub struct App {
    pub event_store: Arc<EventStore>,
    pub timeline_service: Arc<TimelineService>,
    pub nats_client: Arc<NatsClient>,
    pub auth_service: Arc<AuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        // Initialize database connection
        let db_config = DatabaseConfig {
            host: config.database_url.split('@').collect::<Vec<&str>>()[1]
                .split(':').collect::<Vec<&str>>()[0].to_string(),
            port: 5432,
            database: "health_os".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            max_connections: config.max_connections,
        };

        let event_store = Arc::new(EventStore::new(db_config).await?);

        // Initialize NATS client
        let nats_client = Arc::new(NatsClient::new(&config.nats_url).await?);

        // Initialize timeline service
        let timeline_service = Arc::new(TimelineService::new(
            event_store.clone(),
            nats_client.clone(),
        ));

        // Initialize auth service
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
