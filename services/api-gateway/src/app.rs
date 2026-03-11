use auth::AuthService;
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    pub auth_service: Arc<AuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        let auth_service = Arc::new(AuthService::new(
            config.jwt_secret.clone(),
            "health_os".to_string(),
            "health_os_api".to_string(),
        ));

        Ok(Self { auth_service })
    }
}
