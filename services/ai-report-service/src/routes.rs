use axum::{
    routing::{get, post},
    Router,
    middleware,
};

use crate::{app::App, handlers, middleware};

pub fn create_router(app: App) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/reports", post(handlers::generate_report))
        .layer(middleware::from_fn_with_state(app.clone(), middleware::auth_middleware))
        .with_state(app)
}
