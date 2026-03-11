use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

pub async fn login(
    State(app): State<App>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    trace_request!("POST", "/auth/login");
    
    // TODO: Implement actual authentication logic
    // For now, create a mock user and token
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

pub async fn register(
    State(app): State<App>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    trace_request!("POST", "/auth/register");
    
    // TODO: Implement actual registration logic
    // For now, create a mock user and token
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
