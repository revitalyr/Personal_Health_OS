use serde_json::Value;
use anyhow::Result;

pub struct ApiClient {
    base_url: String,
    auth_token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            auth_token: None,
        }
    }

    pub fn set_auth_token(&mut self, token: &str) {
        self.auth_token = Some(token.to_string());
    }

    async fn make_request(&self, endpoint: &str, method: &str, body: Option<Value>) -> Result<Value> {
        let client = reqwest::Client::new();
        let url = format!("{}/{}", self.base_url, endpoint);
        
        let mut request = match method {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
        };

        // Add auth header if available
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // Add body if provided
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            let json: Value = response.json().await?;
            Ok(json)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("API request failed: {}", error_text))
        }
    }

    // Patient Management
    pub async fn get_patients(&self, page: u32, limit: u32) -> Result<Value> {
        let endpoint = format!("/api/v1/patients?page={}&limit={}", page, limit);
        self.make_request(&endpoint, "GET", None).await
    }

    pub async fn create_patient(&self, patient_data: Value) -> Result<Value> {
        self.make_request("/api/v1/patients", "POST", Some(patient_data)).await
    }

    pub async fn get_patient(&self, patient_id: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/patients/{}", patient_id);
        self.make_request(&endpoint, "GET", None).await
    }

    pub async fn update_patient(&self, patient_id: &str, patient_data: Value) -> Result<Value> {
        let endpoint = format!("/api/v1/patients/{}", patient_id);
        self.make_request(&endpoint, "PUT", Some(patient_data)).await
    }

    pub async fn admit_patient(&self, patient_id: &str, admission_data: Value) -> Result<Value> {
        let endpoint = format!("/api/v1/patients/{}/admit", patient_id);
        self.make_request(&endpoint, "POST", Some(admission_data)).await
    }

    pub async fn discharge_patient(&self, patient_id: &str, discharge_data: Value) -> Result<Value> {
        let endpoint = format!("/api/v1/patients/{}/discharge", patient_id);
        self.make_request(&endpoint, "POST", Some(discharge_data)).await
    }

    // Appointment Management
    pub async fn get_appointments(&self, date: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/appointments?date={}", date);
        self.make_request(&endpoint, "GET", None).await
    }

    pub async fn create_appointment(&self, appointment_data: Value) -> Result<Value> {
        self.make_request("/api/v1/appointments", "POST", Some(appointment_data)).await
    }

    pub async fn get_appointment(&self, appointment_id: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/appointments/{}", appointment_id);
        self.make_request(&endpoint, "GET", None).await
    }

    pub async fn update_appointment(&self, appointment_id: &str, appointment_data: Value) -> Result<Value> {
        let endpoint = format!("/api/v1/appointments/{}", appointment_id);
        self.make_request(&endpoint, "PUT", Some(appointment_data)).await
    }

    pub async fn cancel_appointment(&self, appointment_id: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/appointments/{}/cancel", appointment_id);
        self.make_request(&endpoint, "POST", None).await
    }

    // Billing Management
    pub async fn get_invoices(&self, page: u32, limit: u32) -> Result<Value> {
        let endpoint = format!("/api/v1/invoices?page={}&limit={}", page, limit);
        self.make_request(&endpoint, "GET", None).await
    }

    pub async fn create_invoice(&self, invoice_data: Value) -> Result<Value> {
        self.make_request("/api/v1/invoices", "POST", Some(invoice_data)).await
    }

    pub async fn get_invoice(&self, invoice_id: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/invoices/{}", invoice_id);
        self.make_request(&endpoint, "GET", None).await
    }

    pub async fn add_payment(&self, invoice_id: &str, payment_data: Value) -> Result<Value> {
        let endpoint = format!("/api/v1/invoices/{}/payments", invoice_id);
        self.make_request(&endpoint, "POST", Some(payment_data)).await
    }

    // Dashboard Statistics
    pub async fn get_dashboard_stats(&self) -> Result<Value> {
        self.make_request("/api/v1/dashboard/stats", "GET", None).await
    }

    // Service Charges
    pub async fn get_service_charges(&self) -> Result<Value> {
        self.make_request("/api/v1/service-charges", "GET", None).await
    }

    // Facilities
    pub async fn get_facilities(&self) -> Result<Value> {
        self.make_request("/api/v1/facilities", "GET", None).await
    }

    // Doctor Availability
    pub async fn get_doctor_availability(&self, doctor_id: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/doctors/{}/availability", doctor_id);
        self.make_request(&endpoint, "GET", None).await
    }

    // Reports
    pub async fn get_patient_report(&self, patient_id: &str, report_type: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/reports/patients/{}?type={}", patient_id, report_type);
        self.make_request(&endpoint, "GET", None).await
    }

    pub async fn get_financial_report(&self, start_date: &str, end_date: &str) -> Result<Value> {
        let endpoint = format!("/api/v1/reports/financial?start={}&end={}", start_date, end_date);
        self.make_request(&endpoint, "GET", None).await
    }
}
