use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::auth::{AuthService, LoginRequest, RegisterRequest, CreateProfileRequest};
use crate::error::AppError;
use crate::middleware::cors_layer;

pub fn auth_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/login/google", post(login_google))
        .route("/login/apple", post(login_apple))
        .route("/phone/send-otp", post(send_otp))
        .route("/phone/verify", post(verify_otp))
        .route("/profiles", get(get_profiles))
        .route("/profiles", post(create_profile))
        .route("/profiles/:profile_id", get(get_profile))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer()),
        )
}

async fn register(
    State(state): State<crate::AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    let tokens = auth_service.register_email(request).await?;

    Ok(Json(json!({
        "success": true,
        "data": tokens
    })))
}

async fn login(
    State(state): State<crate::AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    let tokens = match request.provider {
        crate::auth::ProviderType::Email => {
            if request.email.is_none() || request.password.is_none() {
                return Err(AppError::BadRequest("Email and password required".to_string()));
            }
            auth_service.login_email(&request.email.unwrap(), &request.password.unwrap()).await?
        }
        crate::auth::ProviderType::Google => {
            if request.token.is_none() {
                return Err(AppError::BadRequest("Google token required".to_string()));
            }
            auth_service.login_google(&request.token.unwrap()).await?
        }
        crate::auth::ProviderType::Apple => {
            if request.token.is_none() {
                return Err(AppError::BadRequest("Apple token required".to_string()));
            }
            auth_service.login_apple(&request.token.unwrap()).await?
        }
        crate::auth::ProviderType::Phone => {
            if request.phone.is_none() || request.otp_code.is_none() {
                return Err(AppError::BadRequest("Phone and OTP code required".to_string()));
            }
            auth_service.verify_otp(&request.phone.unwrap(), &request.otp_code.unwrap()).await?
        }
    };

    Ok(Json(json!({
        "success": true,
        "data": tokens
    })))
}

async fn login_google(
    State(state): State<crate::AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let id_token = payload
        .get("id_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("id_token required".to_string()))?;

    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    let tokens = auth_service.login_google(id_token).await?;

    Ok(Json(json!({
        "success": true,
        "data": tokens
    })))
}

async fn login_apple(
    State(state): State<crate::AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let id_token = payload
        .get("id_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("id_token required".to_string()))?;

    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    let tokens = auth_service.login_apple(id_token).await?;

    Ok(Json(json!({
        "success": true,
        "data": tokens
    })))
}

async fn send_otp(
    State(state): State<crate::AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let phone = payload
        .get("phone")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("phone required".to_string()))?;

    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    auth_service.send_otp(phone).await?;

    Ok(Json(json!({
        "success": true,
        "message": "OTP sent successfully"
    })))
}

async fn verify_otp(
    State(state): State<crate::AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let phone = payload
        .get("phone")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("phone required".to_string()))?;

    let code = payload
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("code required".to_string()))?;

    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    let tokens = auth_service.verify_otp(phone, code).await?;

    Ok(Json(json!({
        "success": true,
        "data": tokens
    })))
}

async fn get_profiles(
    State(state): State<crate::AppState>,
    axum::extract::Extension(account_id): axum::extract::Extension<Uuid>,
) -> Result<Json<Value>, AppError> {
    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    let profiles = auth_service.get_profiles(account_id).await?;

    Ok(Json(json!({
        "success": true,
        "data": profiles
    })))
}

async fn create_profile(
    State(state): State<crate::AppState>,
    axum::extract::Extension(account_id): axum::extract::Extension<Uuid>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<Json<Value>, AppError> {
    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.google_client_id.clone(),
        state.config.apple_client_id.clone(),
    );

    let profile = auth_service.create_profile(account_id, request).await?;

    Ok(Json(json!({
        "success": true,
        "data": profile
    })))
}

async fn get_profile(
    State(_state): State<crate::AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    // Implementation to get specific profile
    Ok(Json(json!({
        "success": true,
        "data": {"id": profile_id}
    })))
}
