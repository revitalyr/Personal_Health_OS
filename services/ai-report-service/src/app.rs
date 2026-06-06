use std::sync::Arc;
use auth::FullAuthService;
use sqlx::PgPool;

#[derive(Clone)]
pub struct App {
    pub auth_service: Arc<FullAuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        // Initialize database pool
        let db = PgPool::connect(&config.database_url).await?;

        // Initialize auth service
        let auth_service = Arc::new(FullAuthService::new(
            db,
            config.jwt_secret.clone(),
            config.google_client_id.clone().unwrap_or_else(|| "".to_string()),
            config.apple_client_id.clone().unwrap_or_else(|| "".to_string()),
        )?);

        tracing::info!("AI Report Service application initialized");

        Ok(Self {
            auth_service,
        })
    }
}
