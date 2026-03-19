use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    // TODO: Add AI service dependencies
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        tracing::info!("AI Report Service application initialized");

        Ok(Self {})
    }
}
