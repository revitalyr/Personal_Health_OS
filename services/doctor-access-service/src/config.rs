use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8084".to_string())
            .parse()
            .unwrap_or(8084);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key".to_string());

        Ok(Config {
            port,
            database_url,
            jwt_secret,
        })
    }
}
