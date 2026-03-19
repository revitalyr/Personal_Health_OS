use axum::{
    routing::{get, post},
    Router,
};

use crate::{app::App, handlers};

pub fn create_router(app: App) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/access/:patient_id", post(handlers::generate_access))
        .route("/view/:token", get(handlers::view_report))
        .with_state(app)
}
