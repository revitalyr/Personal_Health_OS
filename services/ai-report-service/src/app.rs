use std::sync::Arc;
use auth::AuthService;

#[derive(Clone)]
pub struct App {
    pub auth_service: Arc<AuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        // Initialize auth service
        let auth_service = Arc::new(AuthService::new(
            config.jwt_secret.clone(),
            "health_os".to_string(),
            "health_os_api".to_string(),
        )?);

        tracing::info!("AI Report Service application initialized");

        Ok(Self {
            auth_service,
        })
    }
}
