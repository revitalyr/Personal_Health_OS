use axum::{
    routing::{get, post},
    Router,
};

use crate::{app::App, handlers};

pub fn create_router(app: App) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/patients/:patient_id/timeline", get(handlers::timeline::get_timeline))
        .route("/patients/:patient_id/timeline/summary", get(handlers::timeline::get_summary))
        .route("/patients/:patient_id/timeline/export", get(handlers::timeline::export_timeline))
        .route("/patients/:patient_id/events", post(handlers::events::create_event))
        .route("/patients/:patient_id/events/:event_id", get(handlers::events::get_event))
        .route("/patients/:patient_id/events", get(handlers::events::list_events))
        .route("/patients/:patient_id/anomalies", get(handlers::timeline::get_anomalies))
        .layer(middleware::from_fn_with_state(app.clone(), middleware::auth_middleware))
        .with_state(app)
}
