use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::app::App;

pub async fn auth_middleware(
    State(app): State<App>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health check
    let path = request.uri().path();
    if path == "/health" {
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

    // Validate token and extract user_id
    let user_id = app.auth_service.verify_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user_id to request extensions for authorization checks
    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}
