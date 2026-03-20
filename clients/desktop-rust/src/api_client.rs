use tauri::{Manager, State};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// API Client
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    // Authentication
    pub async fn login(&self, email: &str, password: &str) -> anyhow::Result<AuthResponse> {
        let url = format!("{}/auth/login", self.base_url);
        let payload = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Login failed: {}", response.status()))
        }
    }

    // Licence Management
    pub async fn validate_licence(
        &self,
        licence_key: &str,
        hardware_fingerprint: &str,
    ) -> anyhow::Result<LicenceValidationResponse> {
        let url = format!("{}/api/v1/validate", self.base_url);
        let payload = LicenceValidationRequest {
            licence_key: licence_key.to_string(),
            hardware_fingerprint: hardware_fingerprint.to_string(),
            client_id: Uuid::new_v4().to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Licence validation failed: {}", response.status()))
        }
    }

    pub async fn get_licence_status(&self, token: &str) -> anyhow::Result<LicenceStatusResponse> {
        let url = format!("{}/api/v1/licence/status", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get licence status: {}", response.status()))
        }
    }

    // Patient Management
    pub async fn get_patients(
        &self,
        token: &str,
        page: u32,
        limit: u32,
    ) -> anyhow::Result<PatientListResponse> {
        let url = format!("{}/api/v1/patients?page={}&limit={}", self.base_url, page, limit);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get patients: {}", response.status()))
        }
    }

    pub async fn create_patient(
        &self,
        token: &str,
        patient: CreatePatientRequest,
    ) -> anyhow::Result<PatientResponse> {
        let url = format!("{}/api/v1/patients", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&patient)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to create patient: {}", response.status()))
        }
    }

    pub async fn get_patient_timeline(
        &self,
        token: &str,
        patient_id: &str,
        limit: u32,
    ) -> anyhow::Result<PatientTimelineResponse> {
        let url = format!("{}/api/v1/patients/{}/timeline?limit={}", self.base_url, patient_id, limit);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get patient timeline: {}", response.status()))
        }
    }

    // Appointment Management
    pub async fn get_appointments(
        &self,
        token: &str,
        date: &str,
        doctor_id: Option<&str>,
    ) -> anyhow::Result<AppointmentListResponse> {
        let mut url = format!("{}/api/v1/appointments?date={}", self.base_url, date);
        
        if let Some(doctor_id) = doctor_id {
            url.push_str(&format!("&doctor_id={}", doctor_id));
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get appointments: {}", response.status()))
        }
    }

    pub async fn create_appointment(
        &self,
        token: &str,
        appointment: CreateAppointmentRequest,
    ) -> anyhow::Result<AppointmentResponse> {
        let url = format!("{}/api/v1/appointments", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&appointment)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to create appointment: {}", response.status()))
        }
    }

    // Billing & Invoicing
    pub async fn get_invoices(
        &self,
        token: &str,
        page: u32,
        limit: u32,
    ) -> anyhow::Result<InvoiceListResponse> {
        let url = format!("{}/api/v1/invoices?page={}&limit={}", self.base_url, page, limit);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get invoices: {}", response.status()))
        }
    }

    pub async fn create_invoice(
        &self,
        token: &str,
        invoice: CreateInvoiceRequest,
    ) -> anyhow::Result<InvoiceResponse> {
        let url = format!("{}/api/v1/invoices", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&invoice)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to create invoice: {}", response.status()))
        }
    }

    // Dashboard
    pub async fn get_dashboard_stats(&self, token: &str) -> anyhow::Result<DashboardStatsResponse> {
        let url = format!("{}/api/v1/dashboard/stats", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get dashboard stats: {}", response.status()))
        }
    }
}

// Hardware Fingerprint
pub fn generate_hardware_fingerprint() -> anyhow::Result<String> {
    let mut components = Vec::new();
    
    // System information
    if let Ok(hostname) = std::env::var("COMPUTERNAME") {
        components.push(format!("HOSTNAME:{}", hostname));
    }
    
    if let Ok(username) = std::env::var("USERNAME") {
        components.push(format!("USERNAME:{}", username));
    }
    
    // OS information
    components.push(format!("OS:{}", std::env::consts::OS));
    
    // Architecture
    components.push(format!("ARCH:{}", std::env::consts::ARCH));
    
    // Create SHA-256 hash
    let fingerprint = components.join("|");
    let mut hasher = Sha256::new();
    hasher.update(fingerprint.as_bytes());
    let hash = hasher.finalize();
    
    Ok(format!("{:x}", hash))
}

// Data Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenceValidationRequest {
    pub licence_key: String,
    pub hardware_fingerprint: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenceValidationResponse {
    pub licence_id: String,
    pub is_valid: bool,
    pub expiry_date: String,
    pub tier: String,
    pub max_users: u32,
    pub current_users: u32,
    pub features: Vec<String>,
    pub validation_message: String,
    pub next_check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenceStatusResponse {
    pub is_valid: bool,
    pub tier: String,
    pub features: Vec<String>,
    pub expiry_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: String,
    pub patient_id: String,
    pub name: String,
    pub date_of_birth: String,
    pub gender: String,
    pub blood_type: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<EmergencyContact>,
    pub status: String,
    pub admission_date: Option<String>,
    pub discharge_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub name: String,
    pub relationship: String,
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePatientRequest {
    pub patient_id: String,
    pub name: String,
    pub date_of_birth: String,
    pub gender: String,
    pub blood_type: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<EmergencyContact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub doctor_id: String,
    pub doctor_name: String,
    pub facility_id: String,
    pub facility_name: String,
    pub appointment_type: String,
    pub status: String,
    pub start_time: String,
    pub end_time: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAppointmentRequest {
    pub patient_id: String,
    pub doctor_id: String,
    pub facility_id: String,
    pub appointment_type: String,
    pub start_time: String,
    pub end_time: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub invoice_number: String,
    pub patient_id: String,
    pub patient_name: String,
    pub items: Vec<InvoiceItem>,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub status: String,
    pub created_at: String,
    pub due_date: String,
    pub payments: Vec<Payment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub id: String,
    pub service_charge_id: String,
    pub description: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub patient_id: String,
    pub items: Vec<CreateInvoiceItemRequest>,
    pub due_date: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceItemRequest {
    pub service_charge_id: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub invoice_id: String,
    pub amount: f64,
    pub method: String,
    pub status: String,
    pub transaction_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_patients: u32,
    pub today_appointments: u32,
    pub pending_invoices: u32,
    pub monthly_revenue: f64,
    pub active_doctors: u32,
    pub occupied_beds: u32,
    pub total_beds: u32,
}

// Response Wrappers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientListResponse {
    pub data: Vec<Patient>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentListResponse {
    pub data: Vec<Appointment>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListResponse {
    pub data: Vec<Invoice>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientResponse {
    pub data: Patient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentResponse {
    pub data: Appointment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub data: Invoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsResponse {
    pub data: DashboardStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientTimelineResponse {
    pub data: PatientTimeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientTimeline {
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub description: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub total_pages: u32,
}
