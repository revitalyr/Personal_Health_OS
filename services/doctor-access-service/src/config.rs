use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8084".to_string())
            .parse()
            .unwrap_or(8084);

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        Ok(Config {
            port,
            jwt_secret,
        })
    }
}
