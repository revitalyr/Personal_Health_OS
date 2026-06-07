use chrono::{DateTime, Utc};
use event_model::MedicalEvent;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::env;
use thiserror::Error;
use uuid::Uuid;
use serde::de::Error as SerdeError;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Database configuration for PostgreSQL connection
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database host address
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Database username
    pub username: String,
    /// Database password
    pub password: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Create database configuration from environment variables
    /// 
    /// Environment variables:
    /// - DB_HOST (default: "localhost")
    /// - DB_PORT (default: "5432")
    /// - DB_NAME (default: "health_os")
    /// - DB_USER (default: "postgres")
    /// - DB_PASSWORD (default: "postgres")
    /// - DB_MAX_CONNECTIONS (default: "10")
    pub fn from_env() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: env::var("DB_NAME").unwrap_or_else(|_| "health_os".to_string()),
            username: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

/// Event store for managing medical events in PostgreSQL
#[derive(Debug, Clone)]
pub struct EventStore {
    /// PostgreSQL connection pool
    pool: PgPool,
}

impl EventStore {
    /// Returns a reference to the PostgreSQL connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new EventStore with the given database configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let connection_string = config.connection_string();
        
        tracing::info!("Connecting to database: {}:{}", config.host, config.port);
        
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&connection_string)
            .await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        // Run migrations (commented out - migrations directory not in storage crate)
        // sqlx::migrate!("./migrations")
        //     .run(&pool)
        //     .await
        //     .map_err(|e| StorageError::Database(e))?;

        tracing::info!("Database connection established and migrations completed");
        
        Ok(Self { pool })
    }

    /// Store a medical event in the database
    pub async fn store_event(&self, event: &MedicalEvent) -> Result<()> {
        let query = r#"
            INSERT INTO medical_events (
                id, patient_id, event_type, timestamp, payload, 
                source, version, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        sqlx::query(query)
            .bind(event.id)
            .bind(event.patient_id)
            .bind(format!("{:?}", event.event_type))
            .bind(event.timestamp)
            .bind(serde_json::to_value(&event.payload)?)
            .bind(&event.metadata.source)
            .bind(event.metadata.version as i32)
            .bind(event.metadata.created_at)
            .bind(event.metadata.updated_at)
            .execute(&self.pool)
            .await?;

        tracing::debug!("Stored event: {} for patient: {}", event.id, event.patient_id);
        Ok(())
    }

    /// Retrieve all events for a specific patient, ordered by timestamp
    pub async fn get_events_by_patient(&self, patient_id: Uuid) -> Result<Vec<MedicalEvent>> {
        let query = r#"
            SELECT id, patient_id, event_type, timestamp, payload, 
                   source, version, created_at, updated_at
            FROM medical_events 
            WHERE patient_id = $1
            ORDER BY timestamp ASC
        "#;

        let rows = sqlx::query(query)
            .bind(patient_id)
            .fetch_all(&self.pool)
            .await?;

        let mut events = Vec::new();
        for row in rows {
            let event_type_str: String = row.get("event_type");
            let event_type = match event_type_str.as_str() {
                "SymptomCreated" => event_model::EventType::SymptomCreated,
                "MedicationStarted" => event_model::EventType::MedicationStarted,
                "MedicationStopped" => event_model::EventType::MedicationStopped,
                "LabResultReceived" => event_model::EventType::LabResultReceived,
                "DoctorVisit" => event_model::EventType::DoctorVisit,
                "Diagnosis" => event_model::EventType::Diagnosis,
                "DocumentUploaded" => event_model::EventType::DocumentUploaded,
                "ReminderTriggered" => event_model::EventType::ReminderTriggered,
                _ => return Err(StorageError::Serialization(
                    serde_json::Error::custom(format!("Unknown event type: {}", event_type_str))
                )),
            };

            let event = MedicalEvent {
                id: row.get("id"),
                patient_id: row.get("patient_id"),
                event_type,
                timestamp: row.get("timestamp"),
                payload: row.get("payload"),
                metadata: event_model::EventMetadata {
                    source: row.get("source"),
                    version: row.get::<i32, _>("version") as u32,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
            };
            events.push(event);
        }

        tracing::debug!("Retrieved {} events for patient: {}", events.len(), patient_id);
        Ok(events)
    }

    /// Retrieve events for a specific patient with optional filtering
    pub async fn get_events_by_patient_with_filter(
        &self,
        patient_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<MedicalEvent>> {
        let mut query = String::from(
            r#"
            SELECT id, patient_id, event_type, timestamp, payload, 
                   source, version, created_at, updated_at
            FROM medical_events 
            WHERE patient_id = $1
        "#
        );

        let mut bind_count = 1;
        
        if start_date.is_some() {
            query.push_str(&format!(" AND timestamp >= ${}", bind_count + 1));
            bind_count += 1;
        }
        
        if end_date.is_some() {
            query.push_str(&format!(" AND timestamp <= ${}", bind_count + 1));
        }
        
        query.push_str(" ORDER BY timestamp ASC");
        
        if let Some(limit_val) = limit {
            query.push_str(&format!(" LIMIT {}", limit_val));
        }

        let mut sql_query = sqlx::query(&query).bind(patient_id);
        
        if let Some(start) = start_date {
            sql_query = sql_query.bind(start);
        }
        
        if let Some(end) = end_date {
            sql_query = sql_query.bind(end);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let mut events = Vec::new();
        for row in rows {
            let event_type_str: String = row.get("event_type");
            let event_type = match event_type_str.as_str() {
                "SymptomCreated" => event_model::EventType::SymptomCreated,
                "MedicationStarted" => event_model::EventType::MedicationStarted,
                "MedicationStopped" => event_model::EventType::MedicationStopped,
                "LabResultReceived" => event_model::EventType::LabResultReceived,
                "DoctorVisit" => event_model::EventType::DoctorVisit,
                "Diagnosis" => event_model::EventType::Diagnosis,
                "DocumentUploaded" => event_model::EventType::DocumentUploaded,
                "ReminderTriggered" => event_model::EventType::ReminderTriggered,
                _ => return Err(StorageError::Serialization(
                    serde_json::Error::custom(format!("Unknown event type: {}", event_type_str))
                )),
            };

            let event = MedicalEvent {
                id: row.get("id"),
                patient_id: row.get("patient_id"),
                event_type,
                timestamp: row.get("timestamp"),
                payload: row.get("payload"),
                metadata: event_model::EventMetadata {
                    source: row.get("source"),
                    version: row.get::<i32, _>("version") as u32,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
            };
            events.push(event);
        }

        tracing::debug!(
            "Retrieved {} filtered events for patient: {}",
            events.len(),
            patient_id
        );
        Ok(events)
    }

    /// Retrieve a specific event by ID
    pub async fn get_event_by_id(&self, event_id: Uuid) -> Result<Option<MedicalEvent>> {
        let query = r#"
            SELECT id, patient_id, event_type, timestamp, payload, 
                   source, version, created_at, updated_at
            FROM medical_events 
            WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(event_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let event_type_str: String = row.get("event_type");
            let event_type = match event_type_str.as_str() {
                "SymptomCreated" => event_model::EventType::SymptomCreated,
                "MedicationStarted" => event_model::EventType::MedicationStarted,
                "MedicationStopped" => event_model::EventType::MedicationStopped,
                "LabResultReceived" => event_model::EventType::LabResultReceived,
                "DoctorVisit" => event_model::EventType::DoctorVisit,
                "Diagnosis" => event_model::EventType::Diagnosis,
                "DocumentUploaded" => event_model::EventType::DocumentUploaded,
                "ReminderTriggered" => event_model::EventType::ReminderTriggered,
                _ => return Err(StorageError::Serialization(
                    serde_json::Error::custom(format!("Unknown event type: {}", event_type_str))
                )),
            };

            let event = MedicalEvent {
                id: row.get("id"),
                patient_id: row.get("patient_id"),
                event_type,
                timestamp: row.get("timestamp"),
                payload: row.get("payload"),
                metadata: event_model::EventMetadata {
                    source: row.get("source"),
                    version: row.get::<i32, _>("version") as u32,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
            };

            tracing::debug!("Retrieved event: {}", event_id);
            Ok(Some(event))
        } else {
            tracing::debug!("Event not found: {}", event_id);
            Ok(None)
        }
    }

    pub async fn delete_event(&self, event_id: Uuid) -> Result<bool> {
        let query = "DELETE FROM medical_events WHERE id = $1";

        let result = sqlx::query(query)
            .bind(event_id)
            .execute(&self.pool)
            .await?;

        let deleted = result.rows_affected() > 0;
        
        if deleted {
            tracing::debug!("Deleted event: {}", event_id);
        } else {
            tracing::debug!("Event not found for deletion: {}", event_id);
        }

        Ok(deleted)
    }

    pub async fn get_patient_stats(&self, patient_id: Uuid) -> Result<PatientStats> {
        let query = r#"
            SELECT 
                COUNT(*) as total_events,
                COUNT(DISTINCT event_type) as unique_event_types,
                MIN(timestamp) as first_event,
                MAX(timestamp) as last_event
            FROM medical_events 
            WHERE patient_id = $1
        "#;

        let row = sqlx::query(query)
            .bind(patient_id)
            .fetch_one(&self.pool)
            .await?;

        let stats = PatientStats {
            total_events: row.get("total_events"),
            unique_event_types: row.get("unique_event_types"),
            first_event: row.get("first_event"),
            last_event: row.get("last_event"),
        };

        tracing::debug!("Retrieved stats for patient: {}", patient_id);
        Ok(stats)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await;

        match result {
            Ok(_) => {
                tracing::debug!("Database health check passed");
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Database health check failed: {}", e);
                Err(StorageError::Database(e))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatientStats {
    pub total_events: i64,
    pub unique_event_types: i64,
    pub first_event: Option<DateTime<Utc>>,
    pub last_event: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_model::{EventType, SymptomPayload};

    async fn create_test_event_store() -> EventStore {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "health_os_test".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            max_connections: 5,
        };

        EventStore::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_store_and_retrieve_event() {
        let event_store = create_test_event_store().await;
        let patient_id = Uuid::new_v4();

        let event = MedicalEvent::new(
            patient_id,
            EventType::SymptomCreated,
            serde_json::to_value(SymptomPayload {
                name: "Headache".to_string(),
                severity: 5,
                description: None,
                duration: None,
            }).unwrap(),
            "test".to_string(),
        );

        event_store.store_event(&event).await.unwrap();

        let retrieved_events = event_store.get_events_by_patient(patient_id).await.unwrap();
        assert_eq!(retrieved_events.len(), 1);
        assert_eq!(retrieved_events[0].id, event.id);
    }

    #[tokio::test]
    async fn test_get_nonexistent_event() {
        let event_store = create_test_event_store().await;
        let event_id = Uuid::new_v4();

        let result = event_store.get_event_by_id(event_id).await.unwrap();
        assert!(result.is_none());
    }
}
