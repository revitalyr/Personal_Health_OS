use auth::{AuthService, FullAuthService};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    pub auth_service: Arc<AuthService>,
    #[allow(dead_code)]
    pub full_auth_service: Arc<FullAuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        let auth_service = Arc::new(AuthService::new(&config.jwt_secret)?);

        let db = PgPool::connect(&config.database_url).await?;
        let full_auth_service = Arc::new(FullAuthService::new(
            db,
            config.jwt_secret.clone(),
            config.google_client_id.clone(),
            config.apple_client_id.clone(),
        )?);

        Ok(Self { auth_service, full_auth_service })
    }
}
