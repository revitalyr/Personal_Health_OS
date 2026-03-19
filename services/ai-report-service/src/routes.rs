use axum::{
    routing::{get, post},
    Router,
};

use crate::{app::App, handlers};

pub fn create_router(app: App) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/reports", post(handlers::generate_report))
        .with_state(app)
}
