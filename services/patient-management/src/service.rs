use crate::models::*;
use crate::error::{PatientError, Result};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::json;
use validator::Validate;

pub struct PatientService {
    db: PgPool,
}

impl PatientService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // Patient Management
    pub async fn create_patient(&self, request: CreatePatientRequest) -> Result<HospitalPatient> {
        // Validate request
        request.validate()
            .map_err(PatientError::from)?;
        
        // Check if patient ID already exists
        if self.patient_id_exists(&request.patient_id).await? {
            return Err(PatientError::PatientIdExists(request.patient_id));
        }
        
        let now = Utc::now();
        let patient = sqlx::query_as!(
            HospitalPatient,
            r#"
            INSERT INTO hospital_patients (
                profile_id, patient_id, blood_type, emergency_contact_name,
                emergency_contact_phone, insurance_provider, insurance_policy_number,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, profile_id, patient_id, blood_type, emergency_contact_name,
                      emergency_contact_phone, insurance_provider, insurance_policy_number,
                      admission_date, discharge_date,
                      status as "status: PatientStatus",
                      created_at, updated_at
            "#,
            request.profile_id,
            request.patient_id,
            request.blood_type,
            request.emergency_contact_name,
            request.emergency_contact_phone,
            request.insurance_provider,
            request.insurance_policy_number,
            now,
            now
        )
        .fetch_one(&self.db)
        .await?;

        // Log patient creation event
        self.log_patient_event(patient.id, "patient_created", json!({
            "profile_id": request.profile_id,
            "patient_id": request.patient_id
        })).await?;

        Ok(patient)
    }

    pub async fn get_patient(&self, patient_id: PatientId) -> Result<HospitalPatient> {
        sqlx::query_as!(
            HospitalPatient,
            r#"
            SELECT id, profile_id, patient_id, blood_type, emergency_contact_name,
                   emergency_contact_phone, insurance_provider, insurance_policy_number,
                   admission_date, discharge_date,
                   status as "status: PatientStatus",
                   created_at, updated_at
            FROM hospital_patients
            WHERE id = $1
            "#,
            patient_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(PatientError::PatientNotFound(patient_id))
    }

    pub async fn get_patient_by_patient_id(&self, patient_id: &ExternalPatientCode) -> Result<HospitalPatient> {
        sqlx::query_as!(
            HospitalPatient,
            r#"
            SELECT id, profile_id, patient_id, blood_type, emergency_contact_name,
                   emergency_contact_phone, insurance_provider, insurance_policy_number,
                   admission_date, discharge_date,
                   status as "status: PatientStatus",
                   created_at, updated_at
            FROM hospital_patients
            WHERE patient_id = $1
            "#,
            patient_id.to_string()
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(PatientError::PatientNotFound(PatientId::from(Uuid::new_v4()))) // TODO: Get actual ID
    }

    pub async fn update_patient(&self, patient_id: PatientId, request: UpdatePatientRequest) -> Result<HospitalPatient> {
        // Validate request
        request.validate()
            .map_err(PatientError::from)?;
        
        let patient = sqlx::query_as!(
            HospitalPatient,
            r#"
            UPDATE hospital_patients
            SET blood_type = COALESCE($2, blood_type),
                emergency_contact_name = COALESCE($3, emergency_contact_name),
                emergency_contact_phone = COALESCE($4, emergency_contact_phone),
                insurance_provider = COALESCE($5, insurance_provider),
                insurance_policy_number = COALESCE($6, insurance_policy_number),
                updated_at = $7
            WHERE id = $1
            RETURNING id, profile_id, patient_id, blood_type, emergency_contact_name,
                      emergency_contact_phone, insurance_provider, insurance_policy_number,
                      admission_date, discharge_date,
                      status as "status: PatientStatus",
                      created_at, updated_at
            "#,
            patient_id,
            request.blood_type,
            request.emergency_contact_name,
            request.emergency_contact_phone,
            request.insurance_provider,
            request.insurance_policy_number,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log patient update event
        self.log_patient_event(patient_id, "patient_updated", json!({
            "updated_fields": request
        })).await?;

        Ok(patient)
    }

    pub async fn admit_patient(&self, patient_id: Uuid, request: AdmitPatientRequest) -> Result<HospitalPatient, PatientError> {
        let patient = sqlx::query_as!(
            HospitalPatient,
            r#"
            UPDATE hospital_patients
            SET admission_date = $2,
                discharge_date = NULL,
                status = 'active',
                updated_at = $3
            WHERE id = $1
            RETURNING id, profile_id, patient_id, blood_type, emergency_contact_name,
                      emergency_contact_phone, insurance_provider, insurance_policy_number,
                      admission_date, discharge_date,
                      status as "status: PatientStatus",
                      created_at, updated_at
            "#,
            patient_id,
            request.admission_date,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log admission event
        self.log_patient_event(patient_id, "patient_admitted", json!({
            "admission_date": request.admission_date,
            "notes": request.notes
        })).await?;

        Ok(patient)
    }

    pub async fn discharge_patient(&self, patient_id: Uuid, request: DischargePatientRequest) -> Result<HospitalPatient, PatientError> {
        let patient = sqlx::query_as!(
            HospitalPatient,
            r#"
            UPDATE hospital_patients
            SET discharge_date = $2,
                status = 'discharged',
                updated_at = $3
            WHERE id = $1
            RETURNING id, profile_id, patient_id, blood_type, emergency_contact_name,
                      emergency_contact_phone, insurance_provider, insurance_policy_number,
                      admission_date, discharge_date,
                      status as "status: PatientStatus",
                      created_at, updated_at
            "#,
            patient_id,
            request.discharge_date,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log discharge event
        self.log_patient_event(patient_id, "patient_discharged", json!({
            "discharge_date": request.discharge_date,
            "discharge_notes": request.discharge_notes,
            "follow_up_instructions": request.follow_up_instructions
        })).await?;

        Ok(patient)
    }

    pub async fn list_patients(&self, limit: i64, offset: i64, status: Option<PatientStatus>) -> Result<Vec<HospitalPatient>, PatientError> {
        let mut query = "
            SELECT id, profile_id, patient_id, blood_type, emergency_contact_name,
                   emergency_contact_phone, insurance_provider, insurance_policy_number,
                   admission_date, discharge_date,
                   status as \"status: PatientStatus\",
                   created_at, updated_at
            FROM hospital_patients
            WHERE 1=1
        ".to_string();

        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(status) = status {
            query.push_str(&format!(" AND status = ${}", param_index));
            params.push(status);
            param_index += 1;
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_index, param_index + 1));

        let mut query_builder = sqlx::query_as::<_, HospitalPatient>(&query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder = query_builder.bind(limit).bind(offset);

        query_builder.fetch_all(&self.db).await.map_err(PatientError::Database)
    }

    // Medical Encounters
    pub async fn create_encounter(&self, request: CreateEncounterRequest, patient_id: Uuid) -> Result<MedicalEncounter, PatientError> {
        let encounter = sqlx::query_as!(
            MedicalEncounter,
            r#"
            INSERT INTO medical_encounters (
                patient_id, doctor_id, encounter_type, start_time, end_time,
                diagnosis, treatment, notes, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, patient_id, doctor_id, encounter_type as "encounter_type: EncounterType",
                      start_time, end_time, diagnosis, treatment, notes, created_at
            "#,
            patient_id,
            request.doctor_id,
            request.encounter_type,
            request.start_time,
            request.end_time,
            request.diagnosis,
            request.treatment,
            request.notes,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log encounter event
        self.log_patient_event(patient_id, "medical_encounter_created", json!({
            "encounter_id": encounter.id,
            "encounter_type": request.encounter_type,
            "doctor_id": request.doctor_id
        })).await?;

        Ok(encounter)
    }

    pub async fn get_patient_encounters(&self, patient_id: Uuid, limit: i64) -> Result<Vec<MedicalEncounter>, PatientError> {
        sqlx::query_as!(
            MedicalEncounter,
            r#"
            SELECT id, patient_id, doctor_id, encounter_type as "encounter_type: EncounterType",
                   start_time, end_time, diagnosis, treatment, notes, created_at
            FROM medical_encounters
            WHERE patient_id = $1
            ORDER BY start_time DESC
            LIMIT $2
            "#,
            patient_id,
            limit
        )
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)
    }

    // Vitals Management
    pub async fn record_vitals(&self, patient_id: Uuid, request: RecordVitalsRequest, recorded_by: Uuid) -> Result<PatientVitals, PatientError> {
        let vitals = sqlx::query_as!(
            PatientVitals,
            r#"
            INSERT INTO patient_vitals (
                patient_id, blood_pressure_systolic, blood_pressure_diastolic,
                heart_rate, temperature, weight, height, oxygen_saturation,
                recorded_at, recorded_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, patient_id, blood_pressure_systolic, blood_pressure_diastolic,
                      heart_rate, temperature, weight, height, oxygen_saturation,
                      recorded_at, recorded_by
            "#,
            patient_id,
            request.blood_pressure_systolic,
            request.blood_pressure_diastolic,
            request.heart_rate,
            request.temperature,
            request.weight,
            request.height,
            request.oxygen_saturation,
            Utc::now(),
            recorded_by
        )
        .fetch_one(&self.db)
        .await?;

        // Log vitals recording event
        self.log_patient_event(patient_id, "vitals_recorded", json!({
            "vitals_id": vitals.id,
            "recorded_by": recorded_by
        })).await?;

        Ok(vitals)
    }

    pub async fn get_patient_vitals(&self, patient_id: Uuid, limit: i64) -> Result<Vec<PatientVitals>, PatientError> {
        sqlx::query_as!(
            PatientVitals,
            r#"
            SELECT id, patient_id, blood_pressure_systolic, blood_pressure_diastolic,
                   heart_rate, temperature, weight, height, oxygen_saturation,
                   recorded_at, recorded_by
            FROM patient_vitals
            WHERE patient_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2
            "#,
            patient_id,
            limit
        )
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)
    }

    // Allergy Management
    pub async fn add_allergy(&self, patient_id: Uuid, request: AddAllergyRequest) -> Result<PatientAllergy, PatientError> {
        let allergy = sqlx::query_as!(
            PatientAllergy,
            r#"
            INSERT INTO patient_allergies (
                patient_id, allergen, severity, reaction, notes, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, patient_id, allergen, severity as "severity: AllergySeverity",
                      reaction, notes, created_at
            "#,
            patient_id,
            request.allergen,
            request.severity,
            request.reaction,
            request.notes,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log allergy addition event
        self.log_patient_event(patient_id, "allergy_added", json!({
            "allergy_id": allergy.id,
            "allergen": request.allergen,
            "severity": request.severity
        })).await?;

        Ok(allergy)
    }

    pub async fn get_patient_allergies(&self, patient_id: Uuid) -> Result<Vec<PatientAllergy>, PatientError> {
        sqlx::query_as!(
            PatientAllergy,
            r#"
            SELECT id, patient_id, allergen, severity as "severity: AllergySeverity",
                   reaction, notes, created_at
            FROM patient_allergies
            WHERE patient_id = $1
            ORDER BY created_at DESC
            "#,
            patient_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)
    }

    // Medication Management
    pub async fn prescribe_medication(&self, patient_id: Uuid, request: PrescribeMedicationRequest, prescribed_by: Uuid) -> Result<PatientMedication, PatientError> {
        let medication = sqlx::query_as!(
            PatientMedication,
            r#"
            INSERT INTO patient_medications (
                patient_id, medication_name, dosage, frequency, route,
                start_date, end_date, prescribed_by, is_active, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, patient_id, medication_name, dosage, frequency, route,
                      start_date, end_date, prescribed_by, is_active, created_at
            "#,
            patient_id,
            request.medication_name,
            request.dosage,
            request.frequency,
            request.route,
            request.start_date,
            request.end_date,
            prescribed_by,
            true,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log medication prescription event
        self.log_patient_event(patient_id, "medication_prescribed", json!({
            "medication_id": medication.id,
            "medication_name": request.medication_name,
            "prescribed_by": prescribed_by
        })).await?;

        Ok(medication)
    }

    pub async fn get_patient_medications(&self, patient_id: Uuid, active_only: bool) -> Result<Vec<PatientMedication>, PatientError> {
        let query = if active_only {
            sqlx::query_as!(
                PatientMedication,
                r#"
                SELECT id, patient_id, medication_name, dosage, frequency, route,
                       start_date, end_date, prescribed_by, is_active, created_at
                FROM patient_medications
                WHERE patient_id = $1 AND is_active = true
                ORDER BY created_at DESC
                "#,
                patient_id
            )
        } else {
            sqlx::query_as!(
                PatientMedication,
                r#"
                SELECT id, patient_id, medication_name, dosage, frequency, route,
                       start_date, end_date, prescribed_by, is_active, created_at
                FROM patient_medications
                WHERE patient_id = $1
                ORDER BY created_at DESC
                "#,
                patient_id
            )
        };

        query.fetch_all(&self.db).await.map_err(PatientError::Database)
    }

    // Patient Timeline
    pub async fn get_patient_timeline(&self, patient_id: Uuid, limit: i64) -> Result<PatientTimelineResponse, PatientError> {
        // Get all events for the patient
        let encounters = sqlx::query!(
            r#"
            SELECT id, encounter_type as "encounter_type: EncounterType", start_time, diagnosis, notes
            FROM medical_encounters
            WHERE patient_id = $1
            ORDER BY start_time DESC
            LIMIT $2
            "#,
            patient_id,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        let vitals = sqlx::query!(
            r#"
            SELECT id, recorded_at, blood_pressure_systolic, blood_pressure_diastolic, heart_rate, temperature
            FROM patient_vitals
            WHERE patient_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2
            "#,
            patient_id,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        let medications = sqlx::query!(
            r#"
            SELECT id, created_at, medication_name, dosage
            FROM patient_medications
            WHERE patient_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            patient_id,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        let mut events = Vec::new();

        // Convert encounters to timeline events using iterators
        let encounter_events: Vec<TimelineEvent> = encounters.into_iter().map(|encounter| {
            TimelineEvent {
                id: encounter.id,
                event_type: format!("medical_encounter_{}", match encounter.encounter_type {
                    crate::models::EncounterType::Admission => "admission",
                    crate::models::EncounterType::Consultation => "consultation",
                    crate::models::EncounterType::Procedure => "procedure",
                    crate::models::EncounterType::Surgery => "surgery",
                    crate::models::EncounterType::Emergency => "emergency",
                    crate::models::EncounterType::FollowUp => "follow_up",
                    crate::models::EncounterType::Discharge => "discharge",
                }),
                timestamp: encounter.start_time.into(),
                description: encounter.diagnosis
                    .unwrap_or_else(|| Notes::from("Medical encounter".to_string())),
                metadata: json!({
                    "type": encounter.encounter_type,
                    "notes": encounter.notes
                }),
            }
        }).collect();
        events.extend(encounter_events);

        // Convert vitals to timeline events using iterators
        let vital_events: Vec<TimelineEvent> = vitals.into_iter().map(|vital| {
            let systolic = vital.blood_pressure_systolic.unwrap_or(SystolicPressure(0));
            let diastolic = vital.blood_pressure_diastolic.unwrap_or(DiastolicPressure(0));
            let heart_rate = vital.heart_rate.unwrap_or(HeartRate(0));
            let temperature = vital.temperature.unwrap_or(BodyTemperature(0.0));
            
            TimelineEvent {
                id: vital.id,
                event_type: "vitals_recorded".to_string(),
                timestamp: vital.recorded_at.into(),
                description: Notes::from(format!("Vitals recorded: BP {}/{} HR {} T {}°C",
                    systolic.0,
                    diastolic.0,
                    heart_rate.0,
                    temperature.0
                )),
                metadata: json!({
                    "systolic": systolic.0,
                    "diastolic": diastolic.0,
                    "heart_rate": heart_rate.0,
                    "temperature": temperature.0,
                    "weight": vital.weight.map(|w| w.0),
                    "height": vital.height.map(|h| h.0),
                    "oxygen_saturation": vital.oxygen_saturation.map(|o| o.0)
                }),
            }
        }).collect();
        events.extend(vital_events);
        // Convert medications to timeline events using iterators
        let medication_events: Vec<TimelineEvent> = medications.into_iter().map(|medication| {
            TimelineEvent {
                id: medication.id,
                event_type: "medication_prescribed".to_string(),
                timestamp: medication.created_at.into(),
                description: Notes::from(format!("Medication prescribed: {} ({})", 
                    medication.medication_name.0, 
                    medication.dosage.0
                )),
                metadata: json!({
                    "medication_name": medication.medication_name.0,
                    "dosage": medication.dosage.0,
                    "frequency": medication.frequency.0,
                    "route": medication.route
                }),
            }
        }).collect();
        events.extend(medication_events);

        // Sort by timestamp using iterator
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(PatientTimelineResponse {
            patient_id,
            events,
        })
    }

    // Helper methods
    async fn patient_id_exists(&self, patient_id: &ExternalPatientCode) -> Result<bool> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM hospital_patients WHERE patient_id = $1",
            patient_id.to_string()
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count > 0)
    }

    async fn log_patient_event(&self, patient_id: PatientId, event_type: &str, metadata: serde_json::Value) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO patient_events (patient_id, event_type, metadata, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            patient_id,
            event_type,
            metadata,
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(PatientError::Database)?;

        Ok(())
    }
}
