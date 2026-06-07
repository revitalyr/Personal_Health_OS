use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use telemetry::{trace_request, trace_response};

use crate::app::App;

/// Request body for POST /auth/login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub _password: String,
}

/// Request body for POST /auth/register.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub _password: String,
    pub name: String,
}

/// Response body for auth endpoints.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

/// User payload within AuthResponse.
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

/// POST /auth/login — Authenticate a user and return a JWT token.
pub async fn login(
    State(app): State<App>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    trace_request!("POST", "/auth/login");
    
    // TODO: Implement actual authentication logic
    let user_id = Uuid::new_v4();
    
    let user = auth::User {
        id: user_id,
        email: payload.email.clone(),
        name: "Test User".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let token = app.auth_service.generate_token(&user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        },
    };

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(10));
    Ok(Json(response))
}

/// POST /auth/register — Create a new user account and return a JWT token.
pub async fn register(
    State(app): State<App>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    trace_request!("POST", "/auth/register");
    
    // TODO: Implement actual registration logic
    let user_id = Uuid::new_v4();
    
    let user = auth::User {
        id: user_id,
        email: payload.email.clone(),
        name: payload.name,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let token = app.auth_service.generate_token(&user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        },
    };

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}
