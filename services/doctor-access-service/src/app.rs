use auth::AuthService;
use std::sync::Arc;
use crate::config::Config;

#[derive(Clone)]
pub struct App {
    pub auth_service: Arc<AuthService>,
}

impl App {
    pub async fn build(config: &Config) -> anyhow::Result<Self> {
        let auth_service = Arc::new(AuthService::new(&config.jwt_secret)?);

        tracing::info!("Doctor Access Service application initialized");

        Ok(Self {
            auth_service,
        })
    }
}
