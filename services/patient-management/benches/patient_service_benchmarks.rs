//! Benchmarks for the patient management service
//! 
//! These benchmarks measure the performance of key operations
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use patient_management::*;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark setup helper
async fn setup_benchmark_db() -> PgPool {
    let database_url = std::env::var("BENCHMARK_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/benchmark_patient_management".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to benchmark database");
    
    // Clean up any existing data
    sqlx::query("DELETE FROM patient_events")
        .execute(&pool)
        .await
        .expect("Failed to clean up patient events");
    
    sqlx::query("DELETE FROM patient_medications")
        .execute(&pool)
        .await
        .expect("Failed to clean up patient medications");
    
    sqlx::query("DELETE FROM patient_allergies")
        .execute(&pool)
        .await
        .expect("Failed to clean up patient allergies");
    
    sqlx::query("DELETE FROM patient_vitals")
        .execute(&pool)
        .await
        .expect("Failed to clean up patient vitals");
    
    sqlx::query("DELETE FROM medical_encounters")
        .execute(&pool)
        .await
        .expect("Failed to clean up medical encounters");
    
    sqlx::query("DELETE FROM hospital_patients")
        .execute(&pool)
        .await
        .expect("Failed to clean up hospital patients");
    
    pool
}

fn create_test_patient_request(index: usize) -> CreatePatientRequest {
    CreatePatientRequest {
        profile_id: ProfileId::from(Uuid::new_v4()),
        patient_id: ExternalPatientCode::from(format!("BENCH{:04}", index)),
        blood_type: Some(BloodType::OPositive),
        emergency_contact_name: Some(ContactName::from(format!("Contact {}", index))),
        emergency_contact_phone: Some(PhoneNumber::from(format!("555-{:04}-{:04}", index, index))),
        insurance_provider: Some(InsuranceProvider::from("Benchmark Insurance")),
        insurance_policy_number: Some(PolicyNumber::from(format!("POL{:08}", index))),
    }
}

fn create_test_vitals_request() -> RecordVitalsRequest {
    RecordVitalsRequest {
        blood_pressure_systolic: Some(SystolicPressure(120)),
        blood_pressure_diastolic: Some(DiastolicPressure(80)),
        heart_rate: Some(HeartRate(72)),
        temperature: Some(BodyTemperature(36.6)),
        weight: Some(WeightKg(70.5)),
        height: Some(HeightCm(175.0)),
        oxygen_saturation: Some(OxygenSaturation(98.0)),
    }
}

fn create_test_encounter_request() -> CreateEncounterRequest {
    CreateEncounterRequest {
        doctor_id: Some(DoctorId::from(Uuid::new_v4())),
        encounter_type: EncounterType::Consultation,
        start_time: EncounterStart::from(Utc::now()),
        end_time: Some(EncounterEnd::from(Utc::now())),
        diagnosis: Some(Diagnosis::from("Hypertension")),
        treatment: Some(Treatment::from("Lifestyle changes")),
        notes: Some(Notes::from("Routine checkup")),
    }
}

fn benchmark_create_patient(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(setup_benchmark_db());
    let service = Arc::new(PatientService::new(pool));
    
    c.bench_function("create_patient", |b| {
        b.iter(|| {
            let service = Arc::clone(&service);
            let request = create_test_patient_request(black_box(1));
            
            rt.block_on(async move {
                let result = service.create_patient(request).await;
                black_box(result)
            });
        });
    });
    
    // Clean up
    rt.block_on(async move {
        sqlx::query("DELETE FROM hospital_patients")
            .execute(&service.db)
            .await
            .expect("Failed to clean up");
    });
}

fn benchmark_get_patient(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(setup_benchmark_db());
    let service = Arc::new(PatientService::new(pool));
    
    // Create a test patient first
    let patient_id = rt.block_on(async {
        let request = create_test_patient_request(1);
        let patient = service.create_patient(request).await.unwrap();
        patient.id
    });
    
    c.bench_function("get_patient", |b| {
        b.iter(|| {
            let service = Arc::clone(&service);
            let id = black_box(patient_id);
            
            rt.block_on(async move {
                let result = service.get_patient(id).await;
                black_box(result)
            });
        });
    });
    
    // Clean up
    rt.block_on(async move {
        sqlx::query("DELETE FROM hospital_patients")
            .execute(&service.db)
            .await
            .expect("Failed to clean up");
    });
}

fn benchmark_update_patient(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(setup_benchmark_db());
    let service = Arc::new(PatientService::new(pool));
    
    // Create a test patient first
    let patient_id = rt.block_on(async {
        let request = create_test_patient_request(1);
        let patient = service.create_patient(request).await.unwrap();
        patient.id
    });
    
    let update_request = UpdatePatientRequest {
        blood_type: Some(BloodType::APositive),
        emergency_contact_name: Some(ContactName::from("Updated Contact")),
        emergency_contact_phone: Some(PhoneNumber::from("555-999-8888")),
        insurance_provider: Some(InsuranceProvider::from("Updated Insurance")),
        insurance_policy_number: Some(PolicyNumber::from("UPDATED123")),
    };
    
    c.bench_function("update_patient", |b| {
        b.iter(|| {
            let service = Arc::clone(&service);
            let id = black_box(patient_id);
            let request = update_request.clone();
            
            rt.block_on(async move {
                let result = service.update_patient(id, request).await;
                black_box(result)
            });
        });
    });
    
    // Clean up
    rt.block_on(async move {
        sqlx::query("DELETE FROM hospital_patients")
            .execute(&service.db)
            .await
            .expect("Failed to clean up");
    });
}

fn benchmark_record_vitals(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(setup_benchmark_db());
    let service = Arc::new(PatientService::new(pool));
    
    // Create a test patient first
    let patient_id = rt.block_on(async {
        let request = create_test_patient_request(1);
        let patient = service.create_patient(request).await.unwrap();
        patient.id
    });
    
    c.bench_function("record_vitals", |b| {
        b.iter(|| {
            let service = Arc::clone(&service);
            let id = black_box(patient_id);
            let request = create_test_vitals_request();
            
            rt.block_on(async move {
                let result = service.record_vitals(id, request).await;
                black_box(result)
            });
        });
    });
    
    // Clean up
    rt.block_on(async move {
        sqlx::query("DELETE FROM hospital_patients")
            .execute(&service.db)
            .await
            .expect("Failed to clean up");
    });
}

fn benchmark_create_encounter(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(setup_benchmark_db());
    let service = Arc::new(PatientService::new(pool));
    
    // Create a test patient first
    let patient_id = rt.block_on(async {
        let request = create_test_patient_request(1);
        let patient = service.create_patient(request).await.unwrap();
        patient.id
    });
    
    c.bench_function("create_encounter", |b| {
        b.iter(|| {
            let service = Arc::clone(&service);
            let id = black_box(patient_id);
            let request = create_test_encounter_request();
            
            rt.block_on(async move {
                let result = service.create_encounter(id, request).await;
                black_box(result)
            });
        });
    });
    
    // Clean up
    rt.block_on(async move {
        sqlx::query("DELETE FROM hospital_patients")
            .execute(&service.db)
            .await
            .expect("Failed to clean up");
    });
}

fn benchmark_patient_timeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(setup_benchmark_db());
    let service = Arc::new(PatientService::new(pool));
    
    // Create a test patient with multiple events
    let patient_id = rt.block_on(async {
        let request = create_test_patient_request(1);
        let patient = service.create_patient(request).await.unwrap();
        
        // Add multiple encounters
        for i in 0..10 {
            let encounter_request = CreateEncounterRequest {
                doctor_id: Some(DoctorId::from(Uuid::new_v4())),
                encounter_type: EncounterType::Consultation,
                start_time: EncounterStart::from(Utc::now()),
                end_time: Some(EncounterEnd::from(Utc::now())),
                diagnosis: Some(Diagnosis::from(format!("Diagnosis {}", i))),
                treatment: Some(Treatment::from(format!("Treatment {}", i))),
                notes: Some(Notes::from(format!("Notes {}", i))),
            };
            service.create_encounter(patient.id, encounter_request).await.unwrap();
        }
        
        // Add multiple vitals
        for _ in 0..20 {
            let vitals_request = create_test_vitals_request();
            service.record_vitals(patient.id, vitals_request).await.unwrap();
        }
        
        patient.id
    });
    
    c.bench_function("patient_timeline", |b| {
        b.iter(|| {
            let service = Arc::clone(&service);
            let id = black_box(patient_id);
            
            rt.block_on(async move {
                let result = service.get_patient_timeline(id, 100).await;
                black_box(result)
            });
        });
    });
    
    // Clean up
    rt.block_on(async move {
        sqlx::query("DELETE FROM hospital_patients")
            .execute(&service.db)
            .await
            .expect("Failed to clean up");
    });
}

fn benchmark_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(setup_benchmark_db());
    let service = Arc::new(PatientService::new(pool));
    
    let mut group = c.benchmark_group("batch_operations");
    
    // Benchmark creating multiple patients
    group.bench_function("create_100_patients", |b| {
        b.iter(|| {
            let service = Arc::clone(&service);
            
            rt.block_on(async move {
                let mut handles = Vec::new();
                
                for i in 0..100 {
                    let service = Arc::clone(&service);
                    let request = create_test_patient_request(i);
                    
                    let handle = tokio::spawn(async move {
                        service.create_patient(request).await
                    });
                    handles.push(handle);
                }
                
                for handle in handles {
                    black_box(handle.await.unwrap());
                }
            });
        });
    });
    
    // Clean up
    rt.block_on(async move {
        sqlx::query("DELETE FROM hospital_patients")
            .execute(&service.db)
            .await
            .expect("Failed to clean up");
    });
    
    group.finish();
}

fn benchmark_validation(c: &mut Criterion) {
    c.bench_function("validate_patient_request", |b| {
        b.iter(|| {
            let request = create_test_patient_request(black_box(1));
            let result = request.validate();
            black_box(result)
        });
    });
    
    c.bench_function("validate_vitals_request", |b| {
        b.iter(|| {
            let request = create_test_vitals_request();
            let result = request.validate();
            black_box(result)
        });
    });
    
    c.bench_function("validate_encounter_request", |b| {
        b.iter(|| {
            let request = create_test_encounter_request();
            let result = request.validate();
            black_box(result)
        });
    });
}

fn benchmark_type_conversions(c: &mut Criterion) {
    c.bench_function("uuid_to_patient_id_conversion", |b| {
        b.iter(|| {
            let uuid = black_box(Uuid::new_v4());
            let patient_id = PatientId::from(uuid);
            let back_to_uuid = Uuid::from(patient_id);
            black_box(back_to_uuid)
        });
    });
    
    c.bench_function("string_to_external_patient_code", |b| {
        b.iter(|| {
            let code_str = black_box("TEST12345");
            let patient_code = ExternalPatientCode::from(code_str);
            let back_to_string = patient_code.to_string();
            black_box(back_to_string)
        });
    });
    
    c.bench_function("blood_type_display", |b| {
        b.iter(|| {
            let blood_type = black_box(BloodType::OPositive);
            let display_str = blood_type.to_string();
            black_box(display_str)
        });
    });
}

fn benchmark_error_handling(c: &mut Criterion) {
    c.bench_function("create_error", |b| {
        b.iter(|| {
            let error = PatientError::PatientNotFound(black_box(PatientId::from(Uuid::new_v4())));
            let is_client = error.is_client_error();
            let is_server = error.is_server_error();
            let is_retryable = error.is_retryable();
            black_box((is_client, is_server, is_retryable))
        });
    });
    
    c.bench_function("error_display", |b| {
        b.iter(|| {
            let error = PatientError::invalid_data("test_field", "test message");
            let display_str = error.to_string();
            black_box(display_str)
        });
    });
}

criterion_group!(
    benches,
    benchmark_create_patient,
    benchmark_get_patient,
    benchmark_update_patient,
    benchmark_record_vitals,
    benchmark_create_encounter,
    benchmark_patient_timeline,
    benchmark_batch_operations,
    benchmark_validation,
    benchmark_type_conversions,
    benchmark_error_handling
);

criterion_main!(benches);
