use axum::{
    routing::{get, post},
    Router,
};

use crate::{app::App, handlers, middleware};

pub fn create_router(app: App) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/access/:patient_id", post(handlers::generate_access))
        .route("/view/:token", get(handlers::view_report))
        .layer(axum::middleware::from_fn_with_state(app.clone(), middleware::auth_middleware))
        .with_state(app)
}
