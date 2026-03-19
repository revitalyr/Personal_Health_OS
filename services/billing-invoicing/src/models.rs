use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub invoice_number: String,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: f64,
    pub status: InvoiceStatus,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub discount_amount: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Pending,
    Sent,
    Paid,
    Overdue,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvoiceItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub service_charge_id: Option<Uuid>,
    pub description: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
    pub tax_rate: f64,
    pub discount_rate: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub payment_date: DateTime<Utc>,
    pub transaction_id: Option<String>,
    pub status: PaymentStatus,
    pub processor_response: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Card,
    BankTransfer,
    Insurance,
    Online,
    Mobile,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
    PartiallyRefunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServiceCharge {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: ServiceCategory,
    pub base_price: f64,
    pub tax_rate: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum ServiceCategory {
    Consultation,
    Procedure,
    LabTest,
    Imaging,
    Medication,
    Room,
    Emergency,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InsuranceClaim {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub insurance_provider_id: Uuid,
    pub claim_number: String,
    pub status: ClaimStatus,
    pub submitted_amount: f64,
    pub approved_amount: Option<f64>,
    pub denied_amount: Option<f64>,
    pub patient_responsibility: Option<f64>,
    pub notes: Option<String>,
    pub submitted_date: DateTime<Utc>,
    pub processed_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum ClaimStatus {
    Draft,
    Submitted,
    Processing,
    Approved,
    PartiallyApproved,
    Denied,
    Paid,
}

// Request DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvoiceRequest {
    pub patient_id: Uuid,
    #[validate(length(min = 1, max = 50))]
    pub invoice_number: Option<String>,
    pub issue_date: Option<NaiveDate>,
    #[validate(range(min = 1))]
    pub due_days: i32,
    pub items: Vec<CreateInvoiceItemRequest>,
    pub tax_rate: Option<f64>,
    pub discount_rate: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvoiceItemRequest {
    pub service_charge_id: Option<Uuid>,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    #[validate(range(min = 1))]
    pub quantity: i32,
    #[validate(range(min = 0))]
    pub unit_price: f64,
    pub tax_rate: Option<f64>,
    pub discount_rate: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateInvoiceRequest {
    pub issue_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub status: Option<InvoiceStatus>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddPaymentRequest {
    pub invoice_id: Uuid,
    #[validate(range(min = 0.01))]
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub transaction_id: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateServiceChargeRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub category: ServiceCategory,
    #[validate(range(min = 0))]
    pub base_price: f64,
    #[validate(range(min = 0, max = 1))]
    pub tax_rate: f64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubmitInsuranceClaimRequest {
    pub invoice_id: Uuid,
    pub insurance_provider_id: Uuid,
    pub member_id: String,
    pub policy_number: String,
    pub notes: Option<String>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub patient_name: Option<String>,
    pub invoice_number: String,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: f64,
    pub status: InvoiceStatus,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub discount_amount: f64,
    pub balance_due: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<InvoiceItemResponse>,
    pub payments: Vec<PaymentResponse>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceItemResponse {
    pub id: Uuid,
    pub service_charge_id: Option<Uuid>,
    pub service_name: Option<String>,
    pub description: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
    pub tax_rate: f64,
    pub discount_rate: f64,
    pub tax_amount: f64,
    pub discount_amount: f64,
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub payment_date: DateTime<Utc>,
    pub transaction_id: Option<String>,
    pub status: PaymentStatus,
    pub processor_response: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BillingStatistics {
    pub total_invoices: i64,
    pub total_revenue: f64,
    pub outstanding_amount: f64,
    pub overdue_amount: f64,
    pub average_invoice_amount: f64,
    pub invoices_by_status: serde_json::Value,
    pub revenue_by_category: serde_json::Value,
    pub monthly_revenue: Vec<MonthlyRevenue>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyRevenue {
    pub month: String,
    pub revenue: f64,
    pub invoices: i64,
}

#[derive(Debug, Serialize)]
pub struct AgingReport {
    pub current: f64,
    pub days_1_30: f64,
    pub days_31_60: f64,
    pub days_61_90: f64,
    pub over_90: f64,
    pub total_outstanding: f64,
}
