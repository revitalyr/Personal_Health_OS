use axum::{
    routing::{get, post},
    Router,
};

use crate::{app::App, handlers};

pub fn create_router(app: App) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/register", post(handlers::auth::register))
        .route("/patients/:patient_id/timeline", get(handlers::timeline::get_timeline))
        .route("/patients/:patient_id/events", post(handlers::events::create_event))
        .route("/patients/:patient_id/events/:event_id", get(handlers::events::get_event))
        .route("/patients/:patient_id/events", get(handlers::events::list_events))
        .route("/documents/upload", post(handlers::documents::upload_document))
        .route("/ai/reports", post(handlers::ai::generate_report))
        .route("/doctor-access/:patient_id", post(handlers::doctor_access::generate_access))
        .route("/doctor-view/:token", get(handlers::doctor_access::view_report))
        .with_state(app)
}
