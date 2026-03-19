use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Licence {
    pub id: Uuid,
    pub licence_key: String,
    pub customer_name: String,
    pub customer_email: String,
    pub product_name: String,
    pub tier: LicenceTier,
    pub max_users: i32,
    pub current_users: i32,
    pub expiry_date: DateTime<Utc>,
    pub is_active: bool,
    pub hardware_fingerprint: Option<String>,
    pub last_verified: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum LicenceTier {
    Basic,
    Professional,
    Enterprise,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LicenceUsage {
    pub id: Uuid,
    pub licence_id: Uuid,
    pub client_id: String,
    pub action: UsageAction,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum UsageAction {
    LicenceCheck,
    UserLogin,
    FeatureAccess,
    DataExport,
    ReportGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenceValidation {
    pub licence_id: Uuid,
    pub is_valid: bool,
    pub expiry_date: DateTime<Utc>,
    pub tier: LicenceTier,
    pub max_users: i32,
    pub current_users: i32,
    pub features: Vec<String>,
    pub validation_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareFingerprint {
    pub cpu_id: String,
    pub motherboard_id: String,
    pub disk_id: String,
    pub mac_address: String,
    pub os_version: String,
}

// Request DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateLicenceRequest {
    #[validate(length(min = 1, max = 255))]
    pub customer_name: String,
    #[validate(email)]
    pub customer_email: String,
    #[validate(length(min = 1, max = 100))]
    pub product_name: String,
    pub tier: LicenceTier,
    #[validate(range(min = 1, max = 1000))]
    pub max_users: i32,
    #[validate(range(min = 1, max = 3650))]
    pub duration_days: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLicenceRequest {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub tier: Option<LicenceTier>,
    pub max_users: Option<i32>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ValidateLicenceRequest {
    pub licence_key: String,
    pub hardware_fingerprint: Option<HardwareFingerprint>,
    pub client_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReassignLicenceRequest {
    pub new_customer_name: String,
    pub new_customer_email: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct UsageAnalyticsQuery {
    pub licence_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub action: Option<UsageAction>,
    pub group_by: Option<String>, // 'day', 'week', 'month'
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct LicenceResponse {
    pub id: Uuid,
    pub licence_key: String,
    pub customer_name: String,
    pub customer_email: String,
    pub product_name: String,
    pub tier: LicenceTier,
    pub max_users: i32,
    pub current_users: i32,
    pub expiry_date: DateTime<Utc>,
    pub is_active: bool,
    pub days_until_expiry: i64,
    pub last_verified: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UsageAnalytics {
    pub licence_id: Uuid,
    pub customer_name: String,
    pub total_checks: i64,
    pub unique_clients: i64,
    pub active_users: i64,
    pub feature_usage: serde_json::Value,
    pub daily_usage: Vec<DailyUsage>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DailyUsage {
    pub date: String,
    pub checks: i64,
    pub users: i64,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LicenceSummary {
    pub total_licences: i64,
    pub active_licences: i64,
    pub expired_licences: i64,
    pub licences_by_tier: serde_json::Value,
    pub expiring_soon: i64, // Next 30 days
    pub total_users: i64,
    pub revenue_this_month: f64,
}

#[derive(Debug, Serialize)]
pub struct LicenceCheckResponse {
    pub valid: bool,
    pub licence_id: Uuid,
    pub tier: LicenceTier,
    pub expiry_date: DateTime<Utc>,
    pub max_users: i32,
    pub current_users: i32,
    pub features: Vec<String>,
    pub message: String,
    pub next_check: DateTime<Utc>,
}
