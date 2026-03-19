use crate::models::*;
use crate::error::LicenceError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde_json::json;
use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPrivateKey, pkcs8::LineEnding};
use rand::thread_rng;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

pub struct LicenceService {
    db: PgPool,
    private_key: RsaPrivateKey,
}

impl LicenceService {
    pub fn new(db: PgPool, private_key: RsaPrivateKey) -> Self {
        Self { db, private_key }
    }

    // Licence Management
    pub async fn create_licence(&self, request: CreateLicenceRequest) -> Result<Licence, LicenceError> {
        let licence_id = Uuid::new_v4();
        let expiry_date = Utc::now() + Duration::days(request.duration_days as i64);
        
        // Generate licence key
        let licence_key = self.generate_licence_key(
            licence_id,
            &request.customer_email,
            request.tier.clone(),
            request.max_users,
            expiry_date,
        )?;

        let licence = sqlx::query_as!(
            Licence,
            r#"
            INSERT INTO licences (
                id, licence_key, customer_name, customer_email, product_name,
                tier, max_users, current_users, expiry_date, is_active,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, licence_key, customer_name, customer_email, product_name,
                      tier as "tier: LicenceTier", max_users, current_users,
                      expiry_date, is_active, hardware_fingerprint, last_verified,
                      created_at, updated_at
            "#,
            licence_id,
            licence_key,
            request.customer_name,
            request.customer_email,
            request.product_name,
            request.tier,
            request.max_users,
            0, // current_users starts at 0
            expiry_date,
            true,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log licence creation
        self.log_licence_event(licence.id, "licence_created", json!({
            "customer_email": request.customer_email,
            "tier": request.tier,
            "max_users": request.max_users,
            "expiry_date": expiry_date
        })).await?;

        Ok(licence)
    }

    pub async fn get_licence(&self, licence_id: Uuid) -> Result<Licence, LicenceError> {
        sqlx::query_as!(
            Licence,
            r#"
            SELECT id, licence_key, customer_name, customer_email, product_name,
                   tier as "tier: LicenceTier", max_users, current_users,
                   expiry_date, is_active, hardware_fingerprint, last_verified,
                   created_at, updated_at
            FROM licences
            WHERE id = $1
            "#,
            licence_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(LicenceError::LicenceNotFound)
    }

    pub async fn get_licence_by_key(&self, licence_key: &str) -> Result<Licence, LicenceError> {
        sqlx::query_as!(
            Licence,
            r#"
            SELECT id, licence_key, customer_name, customer_email, product_name,
                   tier as "tier: LicenceTier", max_users, current_users,
                   expiry_date, is_active, hardware_fingerprint, last_verified,
                   created_at, updated_at
            FROM licences
            WHERE licence_key = $1
            "#,
            licence_key
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(LicenceError::LicenceNotFound)
    }

    pub async fn update_licence(&self, licence_id: Uuid, request: UpdateLicenceRequest) -> Result<Licence, LicenceError> {
        let licence = sqlx::query_as!(
            Licence,
            r#"
            UPDATE licences
            SET customer_name = COALESCE($2, customer_name),
                customer_email = COALESCE($3, customer_email),
                tier = COALESCE($4, tier),
                max_users = COALESCE($5, max_users),
                expiry_date = COALESCE($6, expiry_date),
                is_active = COALESCE($7, is_active),
                updated_at = $8
            WHERE id = $1
            RETURNING id, licence_key, customer_name, customer_email, product_name,
                      tier as "tier: LicenceTier", max_users, current_users,
                      expiry_date, is_active, hardware_fingerprint, last_verified,
                      created_at, updated_at
            "#,
            licence_id,
            request.customer_name,
            request.customer_email,
            request.tier,
            request.max_users,
            request.expiry_date,
            request.is_active,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log licence update
        self.log_licence_event(licence_id, "licence_updated", json!({
            "updated_fields": request
        })).await?;

        Ok(licence)
    }

    pub async fn suspend_licence(&self, licence_id: Uuid, reason: String) -> Result<Licence, LicenceError> {
        let licence = self.update_licence(
            licence_id,
            UpdateLicenceRequest {
                is_active: Some(false),
                ..Default::default()
            }
        ).await?;

        // Log licence suspension
        self.log_licence_event(licence_id, "licence_suspended", json!({
            "reason": reason
        })).await?;

        Ok(licence)
    }

    pub async fn reactivate_licence(&self, licence_id: Uuid) -> Result<Licence, LicenceError> {
        let licence = self.update_licence(
            licence_id,
            UpdateLicenceRequest {
                is_active: Some(true),
                ..Default::default()
            }
        ).await?;

        // Log licence reactivation
        self.log_licence_event(licence_id, "licence_reactivated", json!({})).await?;

        Ok(licence)
    }

    pub async fn reassign_licence(&self, licence_id: Uuid, request: ReassignLicenceRequest) -> Result<Licence, LicenceError> {
        let licence = self.update_licence(
            licence_id,
            UpdateLicenceRequest {
                customer_name: Some(request.new_customer_name),
                customer_email: Some(request.new_customer_email),
                ..Default::default()
            }
        ).await?;

        // Log licence reassignment
        self.log_licence_event(licence_id, "licence_reassigned", json!({
            "new_customer_name": request.new_customer_name,
            "new_customer_email": request.new_customer_email,
            "reason": request.reason
        })).await?;

        Ok(licence)
    }

    pub async fn list_licences(&self, filters: LicenceFilters) -> Result<Vec<Licence>, LicenceError> {
        let mut query = "
            SELECT id, licence_key, customer_name, customer_email, product_name,
                   tier as \"tier: LicenceTier\", max_users, current_users,
                   expiry_date, is_active, hardware_fingerprint, last_verified,
                   created_at, updated_at
            FROM licences
            WHERE 1=1
        ".to_string();

        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(customer_email) = &filters.customer_email {
            query.push_str(&format!(" AND customer_email ILIKE ${}", param_index));
            params.push(format!("%{}%", customer_email));
            param_index += 1;
        }

        if let Some(tier) = filters.tier {
            query.push_str(&format!(" AND tier = ${}", param_index));
            params.push(tier);
            param_index += 1;
        }

        if let Some(is_active) = filters.is_active {
            query.push_str(&format!(" AND is_active = ${}", param_index));
            params.push(is_active);
            param_index += 1;
        }

        if let Some(expiring_soon) = filters.expiring_soon {
            if expiring_soon {
                query.push_str(&format!(" AND expiry_date <= ${}", param_index));
                params.push(Utc::now() + Duration::days(30));
                param_index += 1;
            }
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT ${}", param_index));
            params.push(limit);
        }

        let mut query_builder = sqlx::query_as::<_, Licence>(&query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }

        query_builder.fetch_all(&self.db).await.map_err(LicenceError::Database)
    }

    // Licence Validation
    pub async fn validate_licence(&self, request: ValidateLicenceRequest) -> Result<LicenceValidation, LicenceError> {
        let licence = self.get_licence_by_key(&request.licence_key).await?;

        // Check if licence is active
        if !licence.is_active {
            return Ok(LicenceValidation {
                licence_id: licence.id,
                is_valid: false,
                expiry_date: licence.expiry_date,
                tier: licence.tier,
                max_users: licence.max_users,
                current_users: licence.current_users,
                features: vec![],
                validation_message: "Licence is suspended".to_string(),
            });
        }

        // Check if licence has expired
        if Utc::now() > licence.expiry_date {
            return Ok(LicenceValidation {
                licence_id: licence.id,
                is_valid: false,
                expiry_date: licence.expiry_date,
                tier: licence.tier,
                max_users: licence.max_users,
                current_users: licence.current_users,
                features: vec![],
                validation_message: "Licence has expired".to_string(),
            });
        }

        // Check hardware fingerprint if provided
        if let (Some(hardware_fp), Some(licence_fp)) = (&request.hardware_fingerprint, &licence.hardware_fingerprint) {
            let current_fingerprint = self.generate_hardware_fingerprint(hardware_fp);
            if current_fingerprint != *licence_fp {
                return Ok(LicenceValidation {
                    licence_id: licence.id,
                    is_valid: false,
                    expiry_date: licence.expiry_date,
                    tier: licence.tier,
                    max_users: licence.max_users,
                    current_users: licence.current_users,
                    features: vec![],
                    validation_message: "Hardware fingerprint mismatch".to_string(),
                });
            }
        }

        // Update last verified timestamp
        sqlx::query!(
            "UPDATE licences SET last_verified = $1 WHERE id = $2",
            Utc::now(),
            licence.id
        )
        .execute(&self.db)
        .await
        .map_err(LicenceError::Database)?;

        // Get features for this tier
        let features = self.get_tier_features(&licence.tier);

        Ok(LicenceValidation {
            licence_id: licence.id,
            is_valid: true,
            expiry_date: licence.expiry_date,
            tier: licence.tier,
            max_users: licence.max_users,
            current_users: licence.current_users,
            features,
            validation_message: "Licence is valid".to_string(),
        })
    }

    // Usage Tracking
    pub async fn record_usage(&self, licence_id: Uuid, client_id: String, action: UsageAction, metadata: serde_json::Value) -> Result<(), LicenceError> {
        sqlx::query!(
            r#"
            INSERT INTO licence_usage (licence_id, client_id, action, timestamp, metadata)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            licence_id,
            client_id,
            action,
            Utc::now(),
            metadata
        )
        .execute(&self.db)
        .await
        .map_err(LicenceError::Database)?;

        Ok(())
    }

    pub async fn get_usage_analytics(&self, query: UsageAnalyticsQuery) -> Result<UsageAnalytics, LicenceError> {
        let licence = if let Some(licence_id) = query.licence_id {
            self.get_licence(licence_id).await?
        } else {
            // Return analytics for first licence if none specified
            self.list_licences(LicenceFilters {
                limit: Some(1),
                ..Default::default()
            }).await?
            .into_iter()
            .next()
            .ok_or(LicenceError::LicenceNotFound)?
        };

        // Get usage statistics
        let usage_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_checks,
                COUNT(DISTINCT client_id) as unique_clients,
                COUNT(DISTINCT DATE(timestamp)) as active_days
            FROM licence_usage
            WHERE licence_id = $1
              AND timestamp >= COALESCE($2, '1970-01-01'::timestamp)
              AND timestamp <= COALESCE($3, NOW())
            "#,
            licence.id,
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(LicenceError::Database)?;

        // Get daily usage
        let daily_usage = sqlx::query!(
            r#"
            SELECT 
                DATE(timestamp) as date,
                COUNT(*) as checks,
                COUNT(DISTINCT client_id) as users
            FROM licence_usage
            WHERE licence_id = $1
              AND timestamp >= COALESCE($2, '1970-01-01'::timestamp)
              AND timestamp <= COALESCE($3, NOW())
            GROUP BY DATE(timestamp)
            ORDER BY date DESC
            "#,
            licence.id,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(LicenceError::Database)?;

        let daily_usage_data: Vec<DailyUsage> = daily_usage.into_iter().map(|row| {
            DailyUsage {
                date: row.date.to_string(),
                checks: row.checks.unwrap_or(0),
                users: row.users.unwrap_or(0),
                features: vec![], // Simplified for now
            }
        }).collect();

        Ok(UsageAnalytics {
            licence_id: licence.id,
            customer_name: licence.customer_name,
            total_checks: usage_stats.total_checks.unwrap_or(0),
            unique_clients: usage_stats.unique_clients.unwrap_or(0),
            active_users: usage_stats.active_days.unwrap_or(0),
            feature_usage: json!({}),
            daily_usage: daily_usage_data,
            period_start: query.start_date.unwrap_or_else(|| Utc::now() - Duration::days(30)),
            period_end: query.end_date.unwrap_or_else(|| Utc::now()),
        })
    }

    pub async fn get_licence_summary(&self) -> Result<LicenceSummary, LicenceError> {
        let summary = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_licences,
                COUNT(*) FILTER (WHERE is_active = true) as active_licences,
                COUNT(*) FILTER (WHERE expiry_date < NOW()) as expired_licences,
                COUNT(*) FILTER (WHERE expiry_date <= NOW() + INTERVAL '30 days' AND expiry_date > NOW()) as expiring_soon,
                SUM(max_users) as total_users
            FROM licences
            "#
        )
        .fetch_one(&self.db)
        .await
        .map_err(LicenceError::Database)?;

        let licences_by_tier = sqlx::query!(
            r#"
            SELECT tier, COUNT(*) as count
            FROM licences
            GROUP BY tier
            "#
        )
        .fetch_all(&self.db)
        .await
        .map_err(LicenceError::Database)?;

        let mut tier_counts = serde_json::Map::new();
        for row in licences_by_tier {
            tier_counts.insert(row.tier, json!(row.count.unwrap_or(0)));
        }

        Ok(LicenceSummary {
            total_licences: summary.total_licences.unwrap_or(0),
            active_licences: summary.active_licences.unwrap_or(0),
            expired_licences: summary.expired_licences.unwrap_or(0),
            licences_by_tier: json!(tier_counts),
            expiring_soon: summary.expiring_soon.unwrap_or(0),
            total_users: summary.total_users.unwrap_or(0),
            revenue_this_month: 0.0, // Would be calculated based on billing
        })
    }

    // Helper methods
    fn generate_licence_key(&self, licence_id: Uuid, email: &str, tier: LicenceTier, max_users: i32, expiry_date: DateTime<Utc>) -> Result<String, LicenceError> {
        let payload = json!({
            "licence_id": licence_id,
            "email": email,
            "tier": tier,
            "max_users": max_users,
            "expiry_date": expiry_date,
            "timestamp": Utc::now()
        });

        let payload_str = serde_json::to_string(&payload).map_err(|e| LicenceError::Validation(e.to_string()))?;
        
        // Create signature
        let mut hasher = Sha256::new();
        hasher.update(payload_str.as_bytes());
        let hash = hasher.finalize();

        // Sign with RSA private key
        let signature = self.private_key
            .sign_blinded(&mut thread_rng(), rsa::PaddingScheme::PKCS1v15Sign { hash: Some(rsa::Hash::SHA2_256) }, &hash)
            .map_err(|e| LicenceError::Validation(e.to_string()))?;

        // Combine payload and signature
        let licence_data = format!("{}.{}", 
            general_purpose::STANDARD.encode(payload_str.as_bytes()),
            general_purpose::STANDARD.encode(signature)
        );

        Ok(licence_data)
    }

    fn generate_hardware_fingerprint(&self, hardware: &HardwareFingerprint) -> String {
        let fingerprint_data = format!("{}{}{}{}{}",
            hardware.cpu_id,
            hardware.motherboard_id,
            hardware.disk_id,
            hardware.mac_address,
            hardware.os_version
        );

        let mut hasher = Sha256::new();
        hasher.update(fingerprint_data.as_bytes());
        let hash = hasher.finalize();

        hex::encode(hash)
    }

    fn get_tier_features(&self, tier: &LicenceTier) -> Vec<String> {
        match tier {
            LicenceTier::Basic => vec![
                "patient_management".to_string(),
                "appointment_scheduling".to_string(),
                "basic_reporting".to_string(),
            ],
            LicenceTier::Professional => vec![
                "patient_management".to_string(),
                "appointment_scheduling".to_string(),
                "billing_invoicing".to_string(),
                "advanced_reporting".to_string(),
                "email_notifications".to_string(),
            ],
            LicenceTier::Enterprise => vec![
                "patient_management".to_string(),
                "appointment_scheduling".to_string(),
                "billing_invoicing".to_string(),
                "advanced_reporting".to_string(),
                "email_notifications".to_string(),
                "sms_notifications".to_string(),
                "api_access".to_string(),
                "multi_facility".to_string(),
                "custom_integrations".to_string(),
            ],
            LicenceTier::Custom => vec![
                "all_features".to_string(),
            ],
        }
    }

    async fn log_licence_event(&self, licence_id: Uuid, event_type: &str, metadata: serde_json::Value) -> Result<(), LicenceError> {
        sqlx::query!(
            r#"
            INSERT INTO licence_events (licence_id, event_type, metadata, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            licence_id,
            event_type,
            metadata,
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(LicenceError::Database)?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LicenceFilters {
    pub customer_email: Option<String>,
    pub tier: Option<LicenceTier>,
    pub is_active: Option<bool>,
    pub expiring_soon: Option<bool>,
    pub limit: Option<i64>,
}
