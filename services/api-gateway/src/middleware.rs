use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_http::cors::CorsLayer;

use crate::app::App;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse(),
            "http://localhost:3001".parse(),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
}

pub async fn auth_middleware(
    State(app): State<App>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health check and login/register endpoints
    let path = request.uri().path();
    if path == "/health" || path.starts_with("/auth/") || path.starts_with("/doctor-view/") {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    app.auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user info to request extensions if needed
    Ok(next.run(request).await)
}
