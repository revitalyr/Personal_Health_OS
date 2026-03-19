use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenceValidation {
    pub licence_id: Uuid,
    pub is_valid: bool,
    pub expiry_date: DateTime<Utc>,
    pub tier: String,
    pub max_users: i32,
    pub current_users: i32,
    pub features: Vec<String>,
    pub validation_message: String,
    pub next_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareFingerprint {
    pub cpu_id: String,
    pub motherboard_id: String,
    pub disk_id: String,
    pub mac_address: String,
    pub os_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLicence {
    pub licence_key: String,
    pub hardware_fingerprint: String,
    pub last_validated: DateTime<Utc>,
}

pub struct LicenceManager {
    current_licence: Option<LicenceValidation>,
    saved_licence: Option<SavedLicence>,
}

impl LicenceManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_licence: None,
            saved_licence: None,
        })
    }

    pub async fn validate_licence(&mut self, licence_key: &str, hardware_fingerprint: &str) -> Result<LicenceValidation> {
        // Call licence validation API
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8088/api/v1/validate")
            .json(&serde_json::json!({
                "licence_key": licence_key,
                "hardware_fingerprint": hardware_fingerprint,
                "client_id": uuid::Uuid::new_v4().to_string()
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let validation: LicenceValidation = response.json().await?;
            self.current_licence = Some(validation.clone());
            Ok(validation)
        } else {
            Ok(LicenceValidation {
                licence_id: Uuid::new_v4(),
                is_valid: false,
                expiry_date: Utc::now(),
                tier: "None".to_string(),
                max_users: 0,
                current_users: 0,
                features: vec![],
                validation_message: "Invalid licence key".to_string(),
                next_check: Utc::now() + chrono::Duration::minutes(30),
            })
        }
    }

    pub fn get_current_licence(&self) -> Option<LicenceValidation> {
        self.current_licence.clone()
    }

    pub fn save_licence(&mut self, licence_key: &str, hardware_fingerprint: &str) -> Result<()> {
        let saved_licence = SavedLicence {
            licence_key: licence_key.to_string(),
            hardware_fingerprint: hardware_fingerprint.to_string(),
            last_validated: Utc::now(),
        };

        // Save to local storage (simplified - in real app would use encrypted storage)
        let licence_json = serde_json::to_string(&saved_licence)?;
        std::fs::write("licence.dat", licence_json)?;
        
        self.saved_licence = Some(saved_licence);
        Ok(())
    }

    pub fn load_saved_licence(&mut self) -> Option<SavedLicence> {
        if let Ok(licence_json) = std::fs::read_to_string("licence.dat") {
            if let Ok(saved_licence) = serde_json::from_str(&licence_json) {
                self.saved_licence = Some(saved_licence.clone());
                return Some(saved_licence);
            }
        }
        None
    }

    pub fn start_periodic_validation(&self) {
        // Start background task to validate licence every 30 minutes
        std::thread::spawn(|| {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1800)); // 30 minutes
                // Validate licence in background
                // This would be implemented with proper async handling
            }
        });
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        if let Some(licence) = &self.current_licence {
            licence.features.contains(&feature.to_string())
        } else {
            false
        }
    }
}

pub fn generate_hardware_fingerprint() -> Result<HardwareFingerprint> {
    // This is a simplified implementation
    // In a real application, you would gather actual hardware information
    
    let cpu_id = get_cpu_id()?;
    let motherboard_id = get_motherboard_id()?;
    let disk_id = get_disk_id()?;
    let mac_address = get_mac_address()?;
    let os_version = get_os_version()?;

    Ok(HardwareFingerprint {
        cpu_id,
        motherboard_id,
        disk_id,
        mac_address,
        os_version,
    })
}

fn get_cpu_id() -> Result<String> {
    // Simplified CPU ID generation
    Ok("CPU-12345".to_string())
}

fn get_motherboard_id() -> Result<String> {
    // Simplified motherboard ID generation
    Ok("MB-67890".to_string())
}

fn get_disk_id() -> Result<String> {
    // Simplified disk ID generation
    Ok("DISK-11111".to_string())
}

fn get_mac_address() -> Result<String> {
    // Simplified MAC address generation
    Ok("00:11:22:33:44:55".to_string())
}

fn get_os_version() -> Result<String> {
    // Get OS version
    Ok(std::env::consts::OS.to_string())
}
