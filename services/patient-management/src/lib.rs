pub mod models;
pub mod service;
pub mod error;
pub mod handlers;
pub mod types;

pub use models::*;
pub use service::*;
pub use error::*;
pub use handlers::*;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
}
