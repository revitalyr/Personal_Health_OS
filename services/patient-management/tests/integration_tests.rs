//! Integration tests for the patient management service
//! 
//! These tests verify the complete functionality of the service
//! including database operations, validation, and error handling.

use patient_management::*;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use validator::Validate;

/// Test setup helper
async fn setup_test_db() -> PgPool {
    // Use environment variable for test database URL
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test_patient_management".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to test database");
    
    // Run migrations if needed
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

/// Clean up test data
async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM patient_events")
        .execute(pool)
        .await
        .expect("Failed to clean up patient events");
    
    sqlx::query("DELETE FROM patient_medications")
        .execute(pool)
        .await
        .expect("Failed to clean up patient medications");
    
    sqlx::query("DELETE FROM patient_allergies")
        .execute(pool)
        .await
        .expect("Failed to clean up patient allergies");
    
    sqlx::query("DELETE FROM patient_vitals")
        .execute(pool)
        .await
        .expect("Failed to clean up patient vitals");
    
    sqlx::query("DELETE FROM medical_encounters")
        .execute(pool)
        .await
        .expect("Failed to clean up medical encounters");
    
    sqlx::query("DELETE FROM hospital_patients")
        .execute(pool)
        .await
        .expect("Failed to clean up hospital patients");
}

#[tokio::test]
async fn test_create_patient_success() {
    let pool = setup_test_db().await;
    let service = PatientService::new(pool.clone());
    
    let request = CreatePatientRequest {
        profile_id: ProfileId::from(Uuid::new_v4()),
        patient_id: ExternalPatientCode::from("TEST001"),
        blood_type: Some(BloodType::OPositive),
        emergency_contact_name: Some(ContactName::from("John Doe")),
        emergency_contact_phone: Some(PhoneNumber::from("555-123-4567")),
        insurance_provider: Some(InsuranceProvider::from("Test Insurance")),
        insurance_policy_number: Some(PolicyNumber::from("POL123456")),
    };
    
    // Validate request before sending
    assert!(request.validate().is_ok());
    
    let result = service.create_patient(request.clone()).await;
    assert!(result.is_ok(), "Failed to create patient: {:?}", result.err());
    
    let patient = result.unwrap();
    assert_eq!(patient.patient_id, request.patient_id);
    assert_eq!(patient.blood_type, request.blood_type);
    assert_eq!(patient.emergency_contact_name, request.emergency_contact_name);
    assert_eq!(patient.emergency_contact_phone, request.emergency_contact_phone);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_patient_duplicate_id() {
    let pool = setup_test_db().await;
    let service = PatientService::new(pool.clone());
    
    let patient_id = ExternalPatientCode::from("DUPLICATE001");
    let profile_id = ProfileId::from(Uuid::new_v4());
    
    // Create first patient
    let request1 = CreatePatientRequest {
        profile_id,
        patient_id: patient_id.clone(),
        blood_type: Some(BloodType::APositive),
        emergency_contact_name: None,
        emergency_contact_phone: None,
        insurance_provider: None,
        insurance_policy_number: None,
    };
    
    service.create_patient(request1).await.unwrap();
    
    // Try to create duplicate
    let request2 = CreatePatientRequest {
        profile_id: ProfileId::from(Uuid::new_v4()),
        patient_id: patient_id.clone(),
        blood_type: Some(BloodType::BPositive),
        emergency_contact_name: None,
        emergency_contact_phone: None,
        insurance_provider: None,
        insurance_policy_number: None,
    };
    
    let result = service.create_patient(request2).await;
    assert!(result.is_err());
    
    match result.err().unwrap() {
        PatientError::PatientIdExists(id) => assert_eq!(id, patient_id),
        other => panic!("Expected PatientIdExists error, got: {:?}", other),
    }
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_patient_success() {
    let pool = setup_test_db().await;
    let service = PatientService::new(pool.clone());
    
    let request = CreatePatientRequest {
        profile_id: ProfileId::from(Uuid::new_v4()),
        patient_id: ExternalPatientCode::from("GET_TEST001"),
        blood_type: Some(BloodType::ONegative),
        emergency_contact_name: Some(ContactName::from("Jane Smith")),
        emergency_contact_phone: Some(PhoneNumber::from("555-987-6543")),
        insurance_provider: Some(InsuranceProvider::from("Health Co")),
        insurance_policy_number: Some(PolicyNumber::from("POL789012")),
    };
    
    let created_patient = service.create_patient(request).await.unwrap();
    
    let retrieved_patient = service.get_patient(created_patient.id).await.unwrap();
    assert_eq!(retrieved_patient.id, created_patient.id);
    assert_eq!(retrieved_patient.patient_id, created_patient.patient_id);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_patient_not_found() {
    let pool = setup_test_db().await;
    let service = PatientService::new(pool.clone());
    
    let non_existent_id = PatientId::from(Uuid::new_v4());
    let result = service.get_patient(non_existent_id).await;
    
    assert!(result.is_err());
    match result.err().unwrap() {
        PatientError::PatientNotFound(id) => assert_eq!(id, non_existent_id),
        other => panic!("Expected PatientNotFound error, got: {:?}", other),
    }
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_patient_success() {
    let pool = setup_test_db().await;
    let service = PatientService::new(pool.clone());
    
    let create_request = CreatePatientRequest {
        profile_id: ProfileId::from(Uuid::new_v4()),
        patient_id: ExternalPatientCode::from("UPDATE_TEST001"),
        blood_type: Some(BloodType::ABPositive),
        emergency_contact_name: Some(ContactName::from("Original Name")),
        emergency_contact_phone: Some(PhoneNumber::from("555-111-2222")),
        insurance_provider: Some(InsuranceProvider::from("Original Insurance")),
        insurance_policy_number: Some(PolicyNumber::from("ORIG123")),
    };
    
    let created_patient = service.create_patient(create_request).await.unwrap();
    
    let update_request = UpdatePatientRequest {
        blood_type: Some(BloodType::ABNegative),
        emergency_contact_name: Some(ContactName::from("Updated Name")),
        emergency_contact_phone: Some(PhoneNumber::from("555-333-4444")),
        insurance_provider: Some(InsuranceProvider::from("Updated Insurance")),
        insurance_policy_number: Some(PolicyNumber::from("UPD456")),
    };
    
    // Validate update request
    assert!(update_request.validate().is_ok());
    
    let updated_patient = service.update_patient(created_patient.id, update_request).await.unwrap();
    
    assert_eq!(updated_patient.id, created_patient.id);
    assert_eq!(updated_patient.patient_id, created_patient.patient_id);
    assert_eq!(updated_patient.blood_type, Some(BloodType::ABNegative));
    assert_eq!(updated_patient.emergency_contact_name, Some(ContactName::from("Updated Name")));
    assert_eq!(updated_patient.emergency_contact_phone, Some(PhoneNumber::from("555-333-4444")));
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_patient_timeline() {
    let pool = setup_test_db().await;
    let service = PatientService::new(pool.clone());
    
    let patient_request = CreatePatientRequest {
        profile_id: ProfileId::from(Uuid::new_v4()),
        patient_id: ExternalPatientCode::from("TIMELINE_TEST001"),
        blood_type: Some(BloodType::OPositive),
        emergency_contact_name: None,
        emergency_contact_phone: None,
        insurance_provider: None,
        insurance_policy_number: None,
    };
    
    let patient = service.create_patient(patient_request).await.unwrap();
    
    // Create medical encounter
    let encounter_request = CreateEncounterRequest {
        doctor_id: Some(DoctorId::from(Uuid::new_v4())),
        encounter_type: EncounterType::Consultation,
        start_time: EncounterStart::from(Utc::now()),
        end_time: Some(EncounterEnd::from(Utc::now())),
        diagnosis: Some(Diagnosis::from("Hypertension")),
        treatment: Some(Treatment::from("Lifestyle changes")),
        notes: Some(Notes::from("Patient education provided")),
    };
    
    service.create_encounter(patient.id, encounter_request).await.unwrap();
    
    // Record vitals
    let vitals_request = RecordVitalsRequest {
        blood_pressure_systolic: Some(SystolicPressure(120)),
        blood_pressure_diastolic: Some(DiastolicPressure(80)),
        heart_rate: Some(HeartRate(72)),
        temperature: Some(BodyTemperature(36.6)),
        weight: Some(WeightKg(70.5)),
        height: Some(HeightCm(175.0)),
        oxygen_saturation: Some(OxygenSaturation(98.0)),
    };
    
    service.record_vitals(patient.id, vitals_request).await.unwrap();
    
    // Get timeline
    let timeline = service.get_patient_timeline(patient.id, 100).await.unwrap();
    
    assert!(!timeline.events.is_empty());
    assert_eq!(timeline.patient_id, patient.id);
    
    // Verify we have both encounter and vitals events
    let event_types: Vec<String> = timeline.events.iter()
        .map(|e| e.event_type.clone())
        .collect();
    
    assert!(event_types.iter().any(|t| t.contains("medical_encounter")));
    assert!(event_types.iter().any(|t| t.contains("vitals_recorded")));
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_phone_number_validation() {
    let valid_phone = PhoneNumber::from("555-123-4567");
    assert!(valid_phone.is_valid());
    
    let invalid_phone = PhoneNumber::from("123");
    assert!(!invalid_phone.is_valid());
    
    let phone_with_letters = PhoneNumber::from("555-ABC-DEFG");
    assert!(!phone_with_letters.is_valid());
}

#[tokio::test]
async fn test_vital_signs_validation() {
    let normal_systolic = SystolicPressure(120);
    assert!(normal_systolic.is_normal());
    
    let high_systolic = SystolicPressure(140);
    assert!(!high_systolic.is_normal());
    
    let normal_diastolic = DiastolicPressure(80);
    assert!(normal_diastolic.is_normal());
    
    let high_diastolic = DiastolicPressure(90);
    assert!(!high_diastolic.is_normal());
    
    let normal_heart_rate = HeartRate(72);
    assert!(normal_heart_rate.is_normal());
    
    let high_heart_rate = HeartRate(120);
    assert!(!high_heart_rate.is_normal());
    
    let normal_temperature = BodyTemperature(36.6);
    assert!(normal_temperature.is_normal());
    
    let high_temperature = BodyTemperature(38.5);
    assert!(!high_temperature.is_normal());
    
    let normal_oxygen = OxygenSaturation(98.0);
    assert!(normal_oxygen.is_normal());
    
    let low_oxygen = OxygenSaturation(92.0);
    assert!(!low_oxygen.is_normal());
}

#[tokio::test]
async fn test_blood_type_display() {
    assert_eq!(BloodType::APositive.to_string(), "A+");
    assert_eq!(BloodType::ANegative.to_string(), "A-");
    assert_eq!(BloodType::OPositive.to_string(), "O+");
    assert_eq!(BloodType::ONegative.to_string(), "O-");
}

#[tokio::test]
async fn test_error_classification() {
    let not_found_error = PatientError::PatientNotFound(PatientId::from(Uuid::new_v4()));
    assert!(not_found_error.is_client_error());
    assert!(!not_found_error.is_server_error());
    assert!(!not_found_error.is_retryable());
    
    let db_error = PatientError::Database(sqlx::Error::Protocol("Connection error".to_string()));
    assert!(!db_error.is_client_error());
    assert!(db_error.is_server_error());
    assert!(db_error.is_retryable());
    
    let validation_error = PatientError::Validation(validator::ValidationErrors::new());
    assert!(validation_error.is_client_error());
    assert!(!validation_error.is_server_error());
    assert!(!validation_error.is_retryable());
    
    let service_unavailable_error = PatientError::service_unavailable("database");
    assert!(!service_unavailable_error.is_client_error());
    assert!(service_unavailable_error.is_server_error());
    assert!(service_unavailable_error.is_retryable());
}

// Property-based tests using proptest
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_phone_number_format(phone in r"[0-9]{3}-[0-9]{3}-[0-9]{4}") {
            let phone_number = PhoneNumber::from(phone);
            prop_assert!(phone_number.is_valid());
        }
        
        #[test]
        fn test_systolic_pressure_range(systolic in 70i32..200) {
            let pressure = SystolicPressure(systolic);
            let is_normal_expected = systolic >= 90 && systolic <= 120;
            prop_assert_eq!(pressure.is_normal(), is_normal_expected);
        }
        
        #[test]
        fn test_diastolic_pressure_range(diastolic in 40i32..120) {
            let pressure = DiastolicPressure(diastolic);
            let is_normal_expected = diastolic >= 60 && diastolic <= 80;
            prop_assert_eq!(pressure.is_normal(), is_normal_expected);
        }
        
        #[test]
        fn test_heart_rate_range(hr in 40i32..200) {
            let heart_rate = HeartRate(hr);
            let is_normal_expected = hr >= 60 && hr <= 100;
            prop_assert_eq!(heart_rate.is_normal(), is_normal_expected);
        }
        
        #[test]
        fn test_temperature_range(temp in 35.0f32..42.0f32) {
            let temperature = BodyTemperature(temp);
            let is_normal_expected = temp >= 36.1 && temp <= 37.2;
            prop_assert_eq!(temperature.is_normal(), is_normal_expected);
        }
        
        #[test]
        fn test_oxygen_saturation_range(o2 in 80.0f32..100.1f32) {
            let saturation = OxygenSaturation(o2);
            let is_normal_expected = o2 >= 95.0 && o2 <= 100.0;
            prop_assert_eq!(saturation.is_normal(), is_normal_expected);
        }
    }
}
