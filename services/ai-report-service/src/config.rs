use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub nats_url: String,
    pub openai_api_key: Option<String>,
    pub ollama_url: Option<String>,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8083".to_string())
            .parse()
            .unwrap_or(8083);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string());

        let nats_url = env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());

        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let ollama_url = env::var("OLLAMA_URL").ok();

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        Ok(Config {
            port,
            database_url,
            nats_url,
            openai_api_key,
            ollama_url,
            jwt_secret,
        })
    }
}
