use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub nats_url: String,
    pub redis_url: Option<String>,
    pub max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8081".to_string())
            .parse()
            .unwrap_or(8081);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string());

        let nats_url = env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());

        let redis_url = env::var("REDIS_URL").ok();

        let max_connections: u32 = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        Ok(Config {
            port,
            database_url,
            nats_url,
            redis_url,
            max_connections,
        })
    }
}
