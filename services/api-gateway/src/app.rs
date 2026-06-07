use auth::AuthService;
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    pub auth_service: Arc<AuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        let auth_service = Arc::new(AuthService::new(&config.jwt_secret)?);

        Ok(Self { auth_service })
    }
}
