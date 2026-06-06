use crate::models::*;
use crate::error::PatientError;
use sqlx::{PgPool, Row};
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

    fn row_to_patient(row: &sqlx::postgres::PgRow) -> crate::error::Result<HospitalPatient> {
        Ok(HospitalPatient {
            id: crate::types::PatientId(row.get::<Uuid, _>("id")),
            profile_id: crate::types::ProfileId(row.get::<Uuid, _>("profile_id")),
            patient_id: crate::types::ExternalPatientCode(row.get::<String, _>("patient_id")),
            blood_type: row.get("blood_type"),
            emergency_contact_name: row.get::<Option<String>, _>("emergency_contact_name").map(|s| crate::types::ContactName(s)),
            emergency_contact_phone: row.get::<Option<String>, _>("emergency_contact_phone").map(|s| crate::types::PhoneNumber(s)),
            insurance_provider: row.get::<Option<String>, _>("insurance_provider").map(|s| crate::types::InsuranceProvider(s)),
            insurance_policy_number: row.get::<Option<String>, _>("insurance_policy_number").map(|s| crate::types::PolicyNumber(s)),
            admission_date: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("admission_date").map(|d| crate::types::AdmissionDate(d)),
            discharge_date: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("discharge_date").map(|d| crate::types::DischargeDate(d)),
            status: row.get("status"),
            created_at: crate::types::CreatedAt(row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")),
            updated_at: crate::types::UpdatedAt(row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at")),
        })
    }

    fn row_to_encounter(row: &sqlx::postgres::PgRow) -> crate::error::Result<MedicalEncounter> {
        Ok(MedicalEncounter {
            id: crate::types::EncounterId(row.get::<Uuid, _>("id")),
            patient_id: crate::types::PatientId(row.get::<Uuid, _>("patient_id")),
            doctor_id: row.get::<Option<Uuid>, _>("doctor_id").map(|d| crate::types::DoctorId(d)),
            encounter_type: match row.get::<String, _>("encounter_type").as_str() {
                "consultation" => crate::models::EncounterType::Consultation,
                "procedure" => crate::models::EncounterType::Procedure,
                "emergency" => crate::models::EncounterType::Emergency,
                "follow_up" => crate::models::EncounterType::FollowUp,
                "admission" => crate::models::EncounterType::Admission,
                "discharge" => crate::models::EncounterType::Discharge,
                "surgery" => crate::models::EncounterType::Surgery,
                _ => return Err(PatientError::Database(sqlx::Error::Decode("Invalid encounter type".into()))),
            },
            start_time: crate::types::EncounterStart(row.get::<chrono::DateTime<chrono::Utc>, _>("start_time")),
            end_time: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("end_time").map(|d| crate::types::EncounterEnd(d)),
            diagnosis: row.get::<Option<String>, _>("diagnosis").map(|d| crate::types::Diagnosis(d)),
            treatment: row.get::<Option<String>, _>("treatment").map(|t| crate::types::Treatment(t)),
            notes: row.get::<Option<String>, _>("notes").map(|n| crate::types::Notes(n)),
            created_at: crate::types::CreatedAt(row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")),
        })
    }

    fn row_to_vitals(row: &sqlx::postgres::PgRow) -> crate::error::Result<PatientVitals> {
        Ok(PatientVitals {
            id: crate::types::VitalsId(row.get::<Uuid, _>("id")),
            patient_id: crate::types::PatientId(row.get::<Uuid, _>("patient_id")),
            blood_pressure_systolic: row.get::<Option<i32>, _>("blood_pressure_systolic").map(|v| crate::types::SystolicPressure(v)),
            blood_pressure_diastolic: row.get::<Option<i32>, _>("blood_pressure_diastolic").map(|v| crate::types::DiastolicPressure(v)),
            heart_rate: row.get::<Option<i32>, _>("heart_rate").map(|v| crate::types::HeartRate(v)),
            temperature: row.get::<Option<f32>, _>("temperature").map(|v| crate::types::BodyTemperature(v)),
            weight: row.get::<Option<f32>, _>("weight").map(|v| crate::types::WeightKg(v)),
            height: row.get::<Option<f32>, _>("height").map(|v| crate::types::HeightCm(v)),
            oxygen_saturation: row.get::<Option<f32>, _>("oxygen_saturation").map(|v| crate::types::OxygenSaturation(v)),
            recorded_at: crate::types::RecordedAt(row.get::<chrono::DateTime<chrono::Utc>, _>("recorded_at")),
            recorded_by: crate::types::StaffId(row.get::<Uuid, _>("recorded_by")),
        })
    }

    fn row_to_allergy(row: &sqlx::postgres::PgRow) -> crate::error::Result<PatientAllergy> {
        Ok(PatientAllergy {
            id: crate::types::AllergyId(row.get::<Uuid, _>("id")),
            patient_id: crate::types::PatientId(row.get::<Uuid, _>("patient_id")),
            allergen: crate::types::AllergenName(row.get::<String, _>("allergen")),
            severity: match row.get::<String, _>("severity").as_str() {
                "mild" => crate::models::AllergySeverity::Mild,
                "moderate" => crate::models::AllergySeverity::Moderate,
                "severe" => crate::models::AllergySeverity::Severe,
                "life_threatening" => crate::models::AllergySeverity::LifeThreatening,
                _ => return Err(PatientError::Database(sqlx::Error::Decode("Invalid allergy severity".into()))),
            },
            reaction: row.get::<Option<String>, _>("reaction").map(|r| crate::types::AllergyReaction(r)),
            notes: row.get::<Option<String>, _>("notes").map(|n| crate::types::AllergyNotes(n)),
            created_at: crate::types::CreatedAt(row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")),
        })
    }

    fn row_to_medication(row: &sqlx::postgres::PgRow) -> crate::error::Result<PatientMedication> {
        Ok(PatientMedication {
            id: crate::types::MedicationRecordId(row.get::<Uuid, _>("id")),
            patient_id: crate::types::PatientId(row.get::<Uuid, _>("patient_id")),
            medication_name: crate::types::MedicationName(row.get::<String, _>("medication_name")),
            dosage: crate::types::Dosage(row.get::<String, _>("dosage")),
            frequency: crate::types::Frequency(row.get::<String, _>("frequency")),
            route: match row.get::<String, _>("route").as_str() {
                "oral" => crate::models::AdministrationRoute::Oral,
                "intravenous" => crate::models::AdministrationRoute::Intravenous,
                "intramuscular" => crate::models::AdministrationRoute::Intramuscular,
                "subcutaneous" => crate::models::AdministrationRoute::Subcutaneous,
                "topical" => crate::models::AdministrationRoute::Topical,
                _ => return Err(PatientError::Database(sqlx::Error::Decode("Invalid administration route".into()))),
            },
            start_date: crate::types::StartDate(row.get::<chrono::DateTime<chrono::Utc>, _>("start_date").date_naive()),
            end_date: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("end_date").map(|d| crate::types::EndDate(d.date_naive())),
            prescribed_by: crate::types::PrescriberId(row.get::<Uuid, _>("prescribed_by")),
            medication_status: match row.get::<bool, _>("is_active") {
                true => crate::models::MedicationStatus::Active,
                false => crate::models::MedicationStatus::Discontinued,
            },
            created_at: crate::types::CreatedAt(row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")),
        })
    }


    // Patient Management
    pub async fn create_patient(&self, request: CreatePatientRequest) -> crate::error::Result<HospitalPatient> {
        // Validate request
        request.validate()
            .map_err(PatientError::from)?;
        
        // Check if patient ID already exists
        if self.patient_id_exists(&request.patient_id).await? {
            return Err(PatientError::PatientIdExists(request.patient_id));
        }
        
        let now = Utc::now();
        let row = sqlx::query(
            r#"INSERT INTO hospital_patients (
                profile_id, patient_id, blood_type, emergency_contact_name,
                emergency_contact_phone, insurance_provider, insurance_policy_number,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, profile_id, patient_id, blood_type, emergency_contact_name,
                      emergency_contact_phone, insurance_provider, insurance_policy_number,
                      admission_date, discharge_date,
                      status,
                      created_at, updated_at
            "#
        )
        .bind(request.profile_id.0)
        .bind(&request.patient_id.0)
        .bind(request.blood_type)
        .bind(request.emergency_contact_name.as_ref().map(|n| n.0.clone()))
        .bind(request.emergency_contact_phone.as_ref().map(|n| n.0.clone()))
        .bind(request.insurance_provider.as_ref().map(|n| n.0.clone()))
        .bind(request.insurance_policy_number.as_ref().map(|n| n.0.clone()))
        .bind(now)
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        let patient = Self::row_to_patient(&row)?;

        // Log patient creation event
        self.log_patient_event(patient.id, "patient_created", json!({
            "profile_id": request.profile_id,
            "patient_id": request.patient_id
        })).await?;

        Ok(patient)
    }

    pub async fn get_patient(&self, patient_id: PatientId) -> crate::error::Result<HospitalPatient> {
        let row = sqlx::query(
            r#"SELECT id, profile_id, patient_id, blood_type, emergency_contact_name,
                   emergency_contact_phone, insurance_provider, insurance_policy_number,
                   admission_date, discharge_date,
                   status,
                   created_at, updated_at
            FROM hospital_patients
            WHERE id = $1"#
        )
        .bind(patient_id.0)
        .fetch_optional(&self.db)
        .await?
        .ok_or(PatientError::PatientNotFound(patient_id))?;

        Ok(Self::row_to_patient(&row)?)
    }

    pub async fn get_patient_by_patient_id(&self, patient_id: &ExternalPatientCode) -> crate::error::Result<HospitalPatient> {
        let row = sqlx::query(
            r#"SELECT id, profile_id, patient_id, blood_type, emergency_contact_name,
                   emergency_contact_phone, insurance_provider, insurance_policy_number,
                   admission_date, discharge_date,
                   status,
                   created_at, updated_at
            FROM hospital_patients
            WHERE patient_id = $1"#
        )
        .bind(patient_id.to_string())
        .fetch_optional(&self.db)
        .await?
        .ok_or(PatientError::PatientNotFound(PatientId::from(Uuid::new_v4())))?;

        Ok(Self::row_to_patient(&row)?)
    }

    pub async fn update_patient(&self, patient_id: PatientId, request: UpdatePatientRequest) -> crate::error::Result<HospitalPatient> {
        // Validate request
        request.validate()
            .map_err(PatientError::from)?;
        
        let row = sqlx::query(
            r#"UPDATE hospital_patients
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
                      status,
                      created_at, updated_at
            "#
        )
        .bind(patient_id.0)
        .bind(request.blood_type)
        .bind(request.emergency_contact_name.as_ref().map(|n| n.0.clone()))
        .bind(request.emergency_contact_phone.as_ref().map(|n| n.0.clone()))
        .bind(request.insurance_provider.as_ref().map(|n| n.0.clone()))
        .bind(request.insurance_policy_number.as_ref().map(|n| n.0.clone()))
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        let patient = Self::row_to_patient(&row)?;

        // Log patient update event
        self.log_patient_event(patient_id, "patient_updated", json!({
            "updated_fields": request
        })).await?;

        Ok(patient)
    }

    pub async fn admit_patient(&self, patient_id: Uuid, request: AdmitPatientRequest) -> crate::error::Result<HospitalPatient> {
        let row = sqlx::query(
            r#"UPDATE hospital_patients
            SET admission_date = $2,
                discharge_date = NULL,
                status = 'active',
                updated_at = $3
            WHERE id = $1
            RETURNING id, profile_id, patient_id, blood_type, emergency_contact_name,
                      emergency_contact_phone, insurance_provider, insurance_policy_number,
                      admission_date, discharge_date,
                      status,
                      created_at, updated_at
            "#
        )
        .bind(patient_id)
        .bind(request.admission_date.0)
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        let patient = Self::row_to_patient(&row)?;

        // Log admission event
        self.log_patient_event(PatientId(patient_id), "patient_admitted", json!({
            "admission_date": request.admission_date,
            "notes": request.notes
        })).await?;

        Ok(patient)
    }

    pub async fn discharge_patient(&self, patient_id: Uuid, request: DischargePatientRequest) -> crate::error::Result<HospitalPatient> {
        let row = sqlx::query(
            r#"UPDATE hospital_patients
            SET discharge_date = $2,
                status = 'discharged',
                updated_at = $3
            WHERE id = $1
            RETURNING id, profile_id, patient_id, blood_type, emergency_contact_name,
                      emergency_contact_phone, insurance_provider, insurance_policy_number,
                      admission_date, discharge_date,
                      status,
                      created_at, updated_at
            "#
        )
        .bind(patient_id)
        .bind(request.discharge_date.0)
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        let patient = Self::row_to_patient(&row)?;

        // Log discharge event
        self.log_patient_event(PatientId(patient_id), "patient_discharged", json!({
            "discharge_date": request.discharge_date,
            "discharge_notes": request.discharge_notes,
            "follow_up_instructions": request.follow_up_instructions
        })).await?;

        Ok(patient)
    }

    pub async fn list_patients(&self, limit: i64, offset: i64, status: Option<PatientStatus>) -> crate::error::Result<Vec<HospitalPatient>> {
        let mut query = "
            SELECT id, profile_id, patient_id, blood_type, emergency_contact_name,
                   emergency_contact_phone, insurance_provider, insurance_policy_number,
                   admission_date, discharge_date,
                   status,
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

        let mut query_builder = sqlx::query(&query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder = query_builder.bind(limit).bind(offset);

        let rows = query_builder.fetch_all(&self.db).await.map_err(PatientError::Database)?;

        rows.iter().map(|row| Self::row_to_patient(row)).collect()
    }

    // Medical Encounters
    pub async fn create_encounter(&self, request: CreateEncounterRequest, patient_id: Uuid) -> crate::error::Result<MedicalEncounter> {
        let now = Utc::now();
        let row = sqlx::query(
            r#"INSERT INTO medical_encounters (
                patient_id, doctor_id, encounter_type, start_time, end_time,
                diagnosis, treatment, notes, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, patient_id, doctor_id, encounter_type,
                      start_time, end_time, diagnosis, treatment, notes, created_at
            "#
        )
        .bind(patient_id)
        .bind(request.doctor_id.as_ref().map(|d| d.0))
        .bind(match request.encounter_type {
            EncounterType::Consultation => "consultation",
            EncounterType::Procedure => "procedure",
            EncounterType::Emergency => "emergency",
            EncounterType::FollowUp => "follow_up",
            EncounterType::Admission => "admission",
            EncounterType::Discharge => "discharge",
            EncounterType::Surgery => "surgery",
        })
        .bind(request.start_time.0)
        .bind(request.end_time.as_ref().map(|t| t.0))
        .bind(request.diagnosis.as_ref().map(|d| d.0.clone()))
        .bind(request.treatment.as_ref().map(|t| t.0.clone()))
        .bind(request.notes.as_ref().map(|n| n.0.clone()))
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        let encounter = Self::row_to_encounter(&row)?;

        // Log encounter event
        self.log_patient_event(PatientId(patient_id), "medical_encounter_created", json!({
            "encounter_id": encounter.id,
            "encounter_type": request.encounter_type,
            "doctor_id": request.doctor_id
        })).await?;

        Ok(encounter)
    }

    pub async fn get_patient_encounters(&self, patient_id: Uuid, limit: i64) -> crate::error::Result<Vec<MedicalEncounter>> {
        let rows = sqlx::query(
            r#"SELECT id, patient_id, doctor_id, encounter_type,
                   start_time, end_time, diagnosis, treatment, notes, created_at
            FROM medical_encounters
            WHERE patient_id = $1
            ORDER BY start_time DESC
            LIMIT $2"#
        )
        .bind(patient_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)?;

        rows.iter().map(|row| Self::row_to_encounter(row)).collect()
    }

    // Vitals Management
    pub async fn record_vitals(&self, patient_id: Uuid, request: RecordVitalsRequest, recorded_by: Uuid) -> crate::error::Result<PatientVitals> {
        let now = Utc::now();
        let row = sqlx::query(
            r#"INSERT INTO patient_vitals (
                patient_id, blood_pressure_systolic, blood_pressure_diastolic,
                heart_rate, temperature, weight, height, oxygen_saturation,
                recorded_at, recorded_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, patient_id, blood_pressure_systolic, blood_pressure_diastolic,
                      heart_rate, temperature, weight, height, oxygen_saturation,
                      recorded_at, recorded_by
            "#
        )
        .bind(patient_id)
        .bind(request.blood_pressure_systolic.map(|v| v.0))
        .bind(request.blood_pressure_diastolic.map(|v| v.0))
        .bind(request.heart_rate.map(|v| v.0))
        .bind(request.temperature.map(|v| v.0))
        .bind(request.weight.map(|v| v.0))
        .bind(request.height.map(|v| v.0))
        .bind(request.oxygen_saturation.map(|v| v.0))
        .bind(now)
        .bind(recorded_by)
        .fetch_one(&self.db)
        .await?;

        let vitals = Self::row_to_vitals(&row)?;

        // Log vitals recording event
        self.log_patient_event(PatientId(patient_id), "vitals_recorded", json!({
            "vitals_id": vitals.id,
            "recorded_by": recorded_by
        })).await?;

        Ok(vitals)
    }

    pub async fn get_patient_vitals(&self, patient_id: Uuid, limit: i64) -> crate::error::Result<Vec<PatientVitals>> {
        let rows = sqlx::query(
            r#"SELECT id, patient_id, blood_pressure_systolic, blood_pressure_diastolic,
                   heart_rate, temperature, weight, height, oxygen_saturation,
                   recorded_at, recorded_by
            FROM patient_vitals
            WHERE patient_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2"#
        )
        .bind(patient_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)?;

        rows.iter().map(|row| Self::row_to_vitals(row)).collect()
    }

    // Allergy Management
    pub async fn add_allergy(&self, patient_id: Uuid, request: AddAllergyRequest) -> crate::error::Result<PatientAllergy> {
        let now = Utc::now();
        let row = sqlx::query(
            r#"INSERT INTO patient_allergies (
                patient_id, allergen, severity, reaction, notes, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, patient_id, allergen, severity,
                      reaction, notes, created_at
            "#
        )
        .bind(patient_id)
        .bind(request.allergen.0.clone())
        .bind(match request.severity {
            AllergySeverity::Mild => "mild",
            AllergySeverity::Moderate => "moderate",
            AllergySeverity::Severe => "severe",
            AllergySeverity::LifeThreatening => "life_threatening",
        })
        .bind(request.reaction.as_ref().map(|r| r.0.clone()))
        .bind(request.notes.as_ref().map(|n| n.0.clone()))
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        let allergy = Self::row_to_allergy(&row)?;

        // Log allergy addition event
        self.log_patient_event(PatientId(patient_id), "allergy_added", json!({
            "allergy_id": allergy.id,
            "allergen": request.allergen,
            "severity": request.severity
        })).await?;

        Ok(allergy)
    }

    pub async fn get_patient_allergies(&self, patient_id: Uuid) -> crate::error::Result<Vec<PatientAllergy>> {
        let rows = sqlx::query(
            r#"SELECT id, patient_id, allergen, severity,
                   reaction, notes, created_at
            FROM patient_allergies
            WHERE patient_id = $1
            ORDER BY created_at DESC"#
        )
        .bind(patient_id)
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)?;

        rows.iter().map(|row| Self::row_to_allergy(row)).collect()
    }

    // Medication Management
    pub async fn prescribe_medication(&self, patient_id: Uuid, request: PrescribeMedicationRequest, prescribed_by: Uuid) -> crate::error::Result<PatientMedication> {
        let now = Utc::now();
        let row = sqlx::query(
            r#"INSERT INTO patient_medications (
                patient_id, medication_name, dosage, frequency, route,
                start_date, end_date, prescribed_by, is_active, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, patient_id, medication_name, dosage, frequency, route,
                      start_date, end_date, prescribed_by, is_active, created_at
            "#
        )
        .bind(patient_id)
        .bind(request.medication_name.0.clone())
        .bind(request.dosage.0.clone())
        .bind(request.frequency.0.clone())
        .bind(request.route)
        .bind(request.start_date.0)
        .bind(request.end_date.as_ref().map(|d| d.0))
        .bind(prescribed_by)
        .bind(true)
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        let medication = Self::row_to_medication(&row)?;

        // Log medication prescription event
        self.log_patient_event(PatientId(patient_id), "medication_prescribed", json!({
            "medication_id": medication.id,
            "medication_name": request.medication_name,
            "prescribed_by": prescribed_by
        })).await?;

        Ok(medication)
    }

    pub async fn get_patient_medications(&self, patient_id: Uuid, active_only: bool) -> crate::error::Result<Vec<PatientMedication>> {
        let query_str = if active_only {
            r#"SELECT id, patient_id, medication_name, dosage, frequency, route,
                       start_date, end_date, prescribed_by, is_active, created_at
                FROM patient_medications
                WHERE patient_id = $1 AND is_active = true
                ORDER BY created_at DESC"#
        } else {
            r#"SELECT id, patient_id, medication_name, dosage, frequency, route,
                       start_date, end_date, prescribed_by, is_active, created_at
                FROM patient_medications
                WHERE patient_id = $1
                ORDER BY created_at DESC"#
        };

        let rows = sqlx::query(query_str)
            .bind(patient_id)
            .fetch_all(&self.db)
            .await
            .map_err(PatientError::Database)?;

        rows.iter().map(|row| Self::row_to_medication(row)).collect()
    }

    // Patient Timeline
    pub async fn get_patient_timeline(&self, patient_id: Uuid, limit: i64) -> crate::error::Result<PatientTimelineResponse> {
        // Get all events for the patient
        let encounters = sqlx::query(
            r#"
            SELECT id, encounter_type, start_time, diagnosis, notes
            FROM medical_encounters
            WHERE patient_id = $1
            ORDER BY start_time DESC
            LIMIT $2
            "#
        )
        .bind(patient_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)?;

        let vitals = sqlx::query(
            r#"
            SELECT id, recorded_at, blood_pressure_systolic, blood_pressure_diastolic, heart_rate, temperature
            FROM patient_vitals
            WHERE patient_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2
            "#
        )
        .bind(patient_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)?;

        let medications = sqlx::query(
            r#"
            SELECT id, created_at, medication_name, dosage
            FROM patient_medications
            WHERE patient_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(patient_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(PatientError::Database)?;

        let mut events = Vec::new();

        // Convert encounters to timeline events using iterators
        let encounter_events: Vec<TimelineEvent> = encounters.into_iter().map(|encounter| {
            let encounter_type_str: String = encounter.get("encounter_type");
            let start_time: chrono::DateTime<chrono::Utc> = encounter.get("start_time");
            let diagnosis: Option<String> = encounter.get("diagnosis");
            let notes: Option<String> = encounter.get("notes");
            
            TimelineEvent {
                id: crate::types::EncounterId(encounter.get::<Uuid, _>("id")),
                event_type: format!("medical_encounter_{}", encounter_type_str),
                timestamp: crate::types::RecordedAt(start_time),
                description: diagnosis.map(|d| crate::types::Notes(d)).unwrap_or_else(|| crate::types::Notes("Medical encounter".to_string())),
                metadata: json!({
                    "type": encounter_type_str,
                    "notes": notes
                }),
            }
        }).collect();
        events.extend(encounter_events);

        // Convert vitals to timeline events using iterators
        let vital_events: Vec<TimelineEvent> = vitals.into_iter().map(|vital| {
            let recorded_at: chrono::DateTime<chrono::Utc> = vital.get("recorded_at");
            let systolic: Option<i32> = vital.get("blood_pressure_systolic");
            let diastolic: Option<i32> = vital.get("blood_pressure_diastolic");
            let heart_rate: Option<i32> = vital.get("heart_rate");
            let temperature: Option<f32> = vital.get("temperature");
            let weight: Option<f32> = vital.get("weight");
            let height: Option<f32> = vital.get("height");
            let oxygen_saturation: Option<f32> = vital.get("oxygen_saturation");
            
            let systolic_val = systolic.unwrap_or(0);
            let diastolic_val = diastolic.unwrap_or(0);
            let heart_rate_val = heart_rate.unwrap_or(0);
            let temperature_val = temperature.unwrap_or(0.0);
            
            TimelineEvent {
                id: crate::types::EncounterId(vital.get::<Uuid, _>("id")),
                event_type: "vitals_recorded".to_string(),
                timestamp: crate::types::RecordedAt(recorded_at),
                description: crate::types::Notes(format!("Vitals recorded: BP {}/{} HR {} T {}°C",
                    systolic_val,
                    diastolic_val,
                    heart_rate_val,
                    temperature_val
                )),
                metadata: json!({
                    "systolic": systolic_val,
                    "diastolic": diastolic_val,
                    "heart_rate": heart_rate_val,
                    "temperature": temperature_val,
                    "weight": weight,
                    "height": height,
                    "oxygen_saturation": oxygen_saturation
                }),
            }
        }).collect();
        events.extend(vital_events);
        // Convert medications to timeline events using iterators
        let medication_events: Vec<TimelineEvent> = medications.into_iter().map(|medication| {
            let created_at: chrono::DateTime<chrono::Utc> = medication.get("created_at");
            let medication_name: String = medication.get("medication_name");
            let dosage: String = medication.get("dosage");
            
            TimelineEvent {
                id: crate::types::EncounterId(medication.get::<Uuid, _>("id")),
                event_type: "medication_prescribed".to_string(),
                timestamp: crate::types::RecordedAt(created_at),
                description: crate::types::Notes(format!("Medication prescribed: {} ({})", 
                    medication_name, 
                    dosage
                )),
                metadata: json!({
                    "medication_name": medication_name,
                    "dosage": dosage
                }),
            }
        }).collect();
        events.extend(medication_events);

        // Sort by timestamp using iterator
        events.sort_by(|a, b| b.timestamp.0.cmp(&a.timestamp.0));

        Ok(PatientTimelineResponse {
            patient_id: crate::types::PatientId(patient_id),
            events,
        })
    }

    // Helper methods
    async fn patient_id_exists(&self, patient_id: &ExternalPatientCode) -> crate::error::Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM hospital_patients WHERE patient_id = $1"
        )
        .bind(patient_id.to_string())
        .fetch_one(&self.db)
        .await?;
        
        Ok(count > 0)
    }

    async fn log_patient_event(&self, patient_id: PatientId, event_type: &str, metadata: serde_json::Value) -> crate::error::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO patient_events (patient_id, event_type, metadata, created_at)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(patient_id.0)
        .bind(event_type)
        .bind(metadata)
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .map_err(PatientError::Database)?;

        Ok(())
    }
}
