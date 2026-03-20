// Standalone Tests for Health OS Semantic Types
// Tests that work independently without compilation issues

use health_os_types::*;

#[cfg(test)]
mod standalone_tests {
    use super::*;

    #[test]
    fn test_basic_semantic_types() {
        // Test basic semantic type creation and validation
        
        // Blood type
        let blood_type = BloodType::OPositive;
        assert_eq!(format!("{:?}", blood_type), "OPositive");
        
        // Patient ID
        let patient_id = new_patient_id();
        let patient_id2 = new_patient_id();
        assert_ne!(patient_id, patient_id2);
        
        // Contact name validation
        assert!(ContactName::new("John Doe").is_ok());
        assert!(ContactName::new("").is_err());
        
        // Phone number validation
        assert!(PhoneNumber::new("555-123-4567").is_ok());
        assert!(PhoneNumber::new("123").is_err());
        
        // Medical measurements
        assert!(SystolicPressure::new(120).is_ok());
        assert!(SystolicPressure::new(300).is_err());
        
        assert!(HeartRate::new(72).is_ok());
        assert!(HeartRate::new(10).is_err());
        
        assert!(BodyTemperature::new(36.6).is_ok());
        assert!(BodyTemperature::new(50.0).is_err());
        
        // Medical content
        assert!(Diagnosis::new("Hypertension").is_ok());
        assert!(Diagnosis::new("").is_err());
        
        assert!(Treatment::new("ACE inhibitors").is_ok());
        assert!(Treatment::new(&"A".repeat(3000)).is_err());
        
        assert!(Notes::new("Patient responded well").is_ok());
        assert!(Notes::new(&"A".repeat(6000)).is_err());
    }

    #[test]
    fn test_blood_pressure_validation() {
        // Valid blood pressure
        let valid = validate_blood_pressure(120, 80);
        assert!(valid.is_ok());
        
        // Invalid (systolic <= diastolic)
        let invalid = validate_blood_pressure(80, 120);
        assert!(invalid.is_err());
        
        // Invalid range
        let invalid_range = validate_blood_pressure(300, 80);
        assert!(invalid_range.is_err());
    }

    #[test]
    fn test_phone_formatting() {
        if let Ok(phone) = PhoneNumber::new("555-123-4567") {
            let formatted = format_phone_number(&phone);
            assert!(formatted.contains("(") || formatted.contains("-"));
        }
    }

    #[test]
    fn test_enum_variants() {
        // Test all enum variants can be created and compared
        
        let blood_types = vec![
            BloodType::APositive, BloodType::ANegative,
            BloodType::BPositive, BloodType::BNegative,
            BloodType::ABPositive, BloodType::ABNegative,
            BloodType::OPositive, BloodType::ONegative,
        ];
        
        let patient_statuses = vec![
            PatientStatus::Admitted, PatientStatus::Discharged,
            PatientStatus::Outpatient, PatientStatus::Emergency,
            PatientStatus::Transferred,
        ];
        
        let encounter_types = vec![
            EncounterType::Consultation, EncounterType::Emergency,
            EncounterType::Surgery, EncounterType::FollowUp,
            EncounterType::Diagnostic, EncounterType::Therapy,
        ];
        
        let allergy_severities = vec![
            AllergySeverity::Mild, AllergySeverity::Moderate,
            AllergySeverity::Severe, AllergySeverity::LifeThreatening,
        ];
        
        let admin_routes = vec![
            AdministrationRoute::Oral, AdministrationRoute::Intravenous,
            AdministrationRoute::Intramuscular, AdministrationRoute::Subcutaneous,
            AdministrationRoute::Topical, AdministrationRoute::Inhalation,
        ];
        
        let med_statuses = vec![
            MedicationStatus::Active, MedicationStatus::Completed,
            MedicationStatus::Discontinued, MedicationStatus::OnHold,
        ];
        
        // Verify all enum variants exist
        assert_eq!(blood_types.len(), 8);
        assert_eq!(patient_statuses.len(), 5);
        assert_eq!(encounter_types.len(), 6);
        assert_eq!(allergy_severities.len(), 4);
        assert_eq!(admin_routes.len(), 6);
        assert_eq!(med_statuses.len(), 4);
    }

    #[test]
    fn test_temporal_types() {
        let created = CreatedAt::now();
        let updated = UpdatedAt::now();
        let recorded = RecordedAt::now();
        
        // Verify temporal types are created and are reasonable
        assert!(created.value() <= Utc::now());
        assert!(updated.value() <= Utc::now());
        assert!(recorded.value() <= Utc::now());
    }

    #[test]
    fn test_patient_creation() {
        let profile_id = ProfileId::new_v4();
        let patient_code = "P12345".to_string();
        
        let patient = HospitalPatient::new(profile_id, patient_code);
        
        assert_eq!(patient.status, PatientStatus::Outpatient);
        assert_eq!(patient.patient_id.as_str(), "P12345");
        assert!(patient.admission_date.is_none());
        assert!(patient.discharge_date.is_none());
    }

    #[test]
    fn test_patient_lifecycle() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        // Admit patient
        assert!(patient.admit().is_ok());
        assert_eq!(patient.status, PatientStatus::Admitted);
        assert!(patient.admission_date.is_some());
        
        // Try to admit again
        assert!(patient.admit().is_err());
        
        // Discharge patient
        assert!(patient.discharge().is_ok());
        assert_eq!(patient.status, PatientStatus::Discharged);
        assert!(patient.discharge_date.is_some());
        
        // Try to discharge again
        assert!(patient.discharge().is_err());
    }

    #[test]
    fn test_emergency_contact() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        // Set valid emergency contact
        assert!(patient.set_emergency_contact("John Doe", "555-123-4567").is_ok());
        assert!(patient.emergency_contact_name.is_some());
        assert!(patient.emergency_contact_phone.is_some());
        
        // Try invalid phone
        assert!(patient.set_emergency_contact("Jane Doe", "123").is_err());
    }

    #[test]
    fn test_insurance_management() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        // Set valid insurance
        assert!(patient.set_insurance("Blue Cross", "BC123456789").is_ok());
        assert!(patient.insurance_provider.is_some());
        assert!(patient.insurance_policy_number.is_some());
        
        // Try empty provider
        assert!(patient.set_insurance("", "POL123").is_err());
    }

    #[test]
    fn test_medical_encounter() {
        let patient_id = PatientId::new_v4();
        let doctor_id = DoctorId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Consultation);
        
        assert_eq!(encounter.patient_id, patient_id);
        assert_eq!(encounter.encounter_type, EncounterType::Consultation);
        assert!(encounter.is_active());
        assert!(encounter.doctor_id.is_none());
        
        // Assign doctor
        encounter.assign_doctor(doctor_id);
        assert_eq!(encounter.doctor_id, Some(doctor_id));
        
        // Set diagnosis and treatment
        assert!(encounter.set_diagnosis("Hypertension").is_ok());
        assert!(encounter.has_diagnosis());
        
        assert!(encounter.set_treatment("Lifestyle changes").is_ok());
        assert!(encounter.has_treatment());
        
        // End encounter
        assert!(encounter.end_encounter().is_ok());
        assert!(!encounter.is_active());
        assert!(encounter.duration().is_some());
        
        // Try to end again
        assert!(encounter.end_encounter().is_err());
    }

    #[test]
    fn test_patient_vitals() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        assert_eq!(vitals.patient_id, patient_id);
        assert_eq!(vitals.recorded_by, staff_id);
        assert!(vitals.blood_pressure_systolic.is_none());
        
        // Set measurements
        assert!(vitals.set_blood_pressure(120, 80).is_ok());
        assert!(vitals.set_heart_rate(72).is_ok());
        assert!(vitals.set_temperature(36.6).is_ok());
        assert!(vitals.set_weight(70.0).is_ok());
        assert!(vitals.set_height(175.0).is_ok());
        assert!(vitals.set_oxygen_saturation(98.0).is_ok());
        
        // Verify measurements were set
        assert!(vitals.blood_pressure_systolic.is_some());
        assert!(vitals.heart_rate.is_some());
        assert!(vitals.temperature.is_some());
        assert!(vitals.weight.is_some());
        assert!(vitals.height.is_some());
        assert!(vitals.oxygen_saturation.is_some());
        
        // Calculate BMI
        let bmi = vitals.calculate_bmi();
        assert!(bmi.is_some());
        
        let expected_bmi = 70.0 / ((175.0 / 100.0) * (175.0 / 100.0));
        assert!((bmi.unwrap() - expected_bmi).abs() < 0.1);
        
        // Check normal ranges
        assert_eq!(vitals.is_blood_pressure_normal(), Some(true));
        assert_eq!(vitals.is_heart_rate_normal(), Some(true));
        assert_eq!(vitals.is_temperature_normal(), Some(true));
        assert_eq!(vitals.is_oxygen_saturation_normal(), Some(true));
    }

    #[test]
    fn test_patient_allergy() {
        let patient_id = PatientId::new_v4();
        let allergy = PatientAllergy::new(
            patient_id,
            "Peanuts",
            AllergySeverity::Severe,
        );
        
        assert!(allergy.is_ok());
        let allergy = allergy.unwrap();
        assert_eq!(allergy.allergen.as_str(), "Peanuts");
        assert_eq!(allergy.severity, AllergySeverity::Severe);
        assert!(allergy.is_severe());
        assert!(!allergy.is_life_threatening());
        
        // Test severity levels
        let mild = PatientAllergy::new(patient_id, "Dust", AllergySeverity::Mild).unwrap();
        assert!(!mild.is_severe());
        assert!(!mild.is_life_threatening());
        
        let life_threatening = PatientAllergy::new(patient_id, "Peanuts", AllergySeverity::LifeThreatening).unwrap();
        assert!(life_threatening.is_severe());
        assert!(life_threatening.is_life_threatening());
        
        // Test allergy categories
        let medication = PatientAllergy::new(patient_id, "Penicillin", AllergySeverity::Moderate).unwrap();
        assert_eq!(medication.get_allergy_category(), AllergyCategory::Medication);
        
        let food = PatientAllergy::new(patient_id, "Shellfish", AllergySeverity::Severe).unwrap();
        assert_eq!(food.get_allergy_category(), AllergyCategory::Food);
        
        let environmental = PatientAllergy::new(patient_id, "Pollen", AllergySeverity::Mild).unwrap();
        assert_eq!(environmental.get_allergy_category(), AllergyCategory::Environmental);
        
        let material = PatientAllergy::new(patient_id, "Latex", AllergySeverity::Moderate).unwrap();
        assert_eq!(material.get_allergy_category(), AllergyCategory::Material);
        
        // Test emergency actions
        let mild_actions = mild.get_emergency_actions();
        assert!(mild_actions.contains(&"Monitor for symptoms"));
        
        let severe_actions = allergy.get_emergency_actions();
        assert!(severe_actions.contains(&"Call emergency services"));
        
        let lt_actions = life_threatening.get_emergency_actions();
        assert!(lt_actions.contains(&"EMERGENCY: Call 911 immediately"));
    }

    #[test]
    fn test_patient_medication() {
        use chrono::NaiveDate;
        
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        let medication = PatientMedication::new(
            patient_id,
            "Lisinopril",
            "10mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        );
        
        assert!(medication.is_ok());
        let med = medication.unwrap();
        assert_eq!(med.medication_name.as_str(), "Lisinopril");
        assert_eq!(med.dosage.as_str(), "10mg");
        assert_eq!(med.frequency.as_str(), "Once daily");
        assert_eq!(med.route, AdministrationRoute::Oral);
        assert!(med.is_active());
        assert!(!med.requires_monitoring());
        
        // Test medication lifecycle
        let mut med_active = PatientMedication::new(
            patient_id,
            "Amoxicillin",
            "500mg",
            "Three times daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        
        // Initial state
        assert!(med_active.is_active());
        assert!(!med_active.is_completed());
        
        // Put on hold
        assert!(med_active.put_on_hold().is_ok());
        assert!(med_active.is_on_hold());
        assert!(!med_active.is_active());
        
        // Resume from hold
        assert!(med_active.resume_medication().is_ok());
        assert!(med_active.is_active());
        assert!(!med_active.is_on_hold());
        
        // Complete medication
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 22).unwrap();
        assert!(med_active.end_medication(end_date).is_ok());
        assert!(med_active.is_completed());
        assert!(!med_active.is_active());
        
        // Test medication routes
        let oral = PatientMedication::new(
            patient_id,
            "Aspirin",
            "81mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        assert_eq!(oral.route_display(), "Oral");
        
        let iv = PatientMedication::new(
            patient_id,
            "Morphine",
            "2mg",
            "As needed",
            AdministrationRoute::Intravenous,
            start_date,
            prescriber_id,
        ).unwrap();
        assert_eq!(iv.route_display(), "Intravenous");
        assert!(iv.requires_monitoring());
        
        let im = PatientMedication::new(
            patient_id,
            "Vitamin B12",
            "1000mcg",
            "Monthly",
            AdministrationRoute::Intramuscular,
            start_date,
            prescriber_id,
        ).unwrap();
        assert_eq!(im.route_display(), "Intramuscular");
        assert!(im.requires_monitoring());
    }

    #[test]
    fn test_create_patient_request() {
        let profile_id = ProfileId::new_v4();
        
        // Valid request
        let valid_request = CreatePatientRequest {
            profile_id,
            patient_id: "P12345".to_string(),
            blood_type: Some(BloodType::OPositive),
            emergency_contact_name: Some("John Doe".to_string()),
            emergency_contact_phone: Some("555-123-4567".to_string()),
            insurance_provider: Some("Blue Cross".to_string()),
            insurance_policy_number: Some("BC123456789".to_string()),
        };

        assert!(valid_request.validate_request().is_ok());
        
        let patient = valid_request.to_hospital_patient();
        assert!(patient.is_ok());
        
        let patient = patient.unwrap();
        assert_eq!(patient.profile_id, profile_id);
        assert_eq!(patient.patient_id.as_str(), "P12345");
        assert_eq!(patient.blood_type, Some(BloodType::OPositive));
    }

    #[test]
    fn test_invalid_patient_request() {
        let profile_id = ProfileId::new_v4();
        
        // Invalid phone number
        let invalid_request = CreatePatientRequest {
            profile_id,
            patient_id: "P12345".to_string(),
            blood_type: Some(BloodType::OPositive),
            emergency_contact_name: Some("John Doe".to_string()),
            emergency_contact_phone: Some("123".to_string()), // Too short
            insurance_provider: Some("Blue Cross".to_string()),
            insurance_policy_number: Some("BC123456789".to_string()),
        };

        assert!(invalid_request.validate_request().is_err());
        assert!(invalid_request.to_hospital_patient().is_err());
    }

    #[test]
    fn test_semantic_type_benefits() {
        // This test demonstrates the benefits of semantic types
        
        // 1. Type Safety - Invalid data caught at creation time
        let valid_phone = PhoneNumber::new("555-123-4567").unwrap();
        let invalid_phone = PhoneNumber::new("123");
        
        assert!(valid_phone.as_str().len() > 0);
        assert!(invalid_phone.is_err());
        
        // 2. Self-documenting code
        let blood_pressure = SystolicPressure::new(120).unwrap();
        let temperature = BodyTemperature::new(36.6).unwrap();
        
        // Clear what these represent without comments
        assert_eq!(blood_pressure.value(), 120);
        assert_eq!(temperature.value(), 36.6);
        
        // 3. Built-in validation
        let valid_weight = WeightKg::new(70.0).unwrap();
        let invalid_weight = WeightKg::new(1000.0); // Too heavy
        
        assert!(valid_weight.value() == 70.0);
        assert!(invalid_weight.is_err());
        
        // 4. Semantic methods
        let formatted_phone = format_phone_number(&valid_phone);
        assert!(formatted_phone.contains("(") || formatted_phone.contains("-"));
        
        // 5. Business logic integration
        let (systolic, diastolic) = validate_blood_pressure(120, 80).unwrap();
        assert_eq!(systolic.value(), 120);
        assert_eq!(diastolic.value(), 80);
        
        // Invalid blood pressure validation
        let invalid_bp = validate_blood_pressure(80, 120); // systolic < diastolic
        assert!(invalid_bp.is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        // Test that semantic types can be serialized and deserialized
        let patient_id = PatientId::new_v4();
        let blood_type = BloodType::ABNegative;
        
        // Serialize
        let patient_serialized = serde_json::to_string(&patient_id).unwrap();
        let blood_type_serialized = serde_json::to_string(&blood_type).unwrap();
        
        // Deserialize
        let patient_deserialized: PatientId = serde_json::from_str(&patient_serialized).unwrap();
        let blood_type_deserialized: BloodType = serde_json::from_str(&blood_type_serialized).unwrap();
        
        // Verify roundtrip
        assert_eq!(patient_id, patient_deserialized);
        assert_eq!(blood_type, blood_type_deserialized);
    }

    #[test]
    fn test_error_handling_consistency() {
        // Test that all semantic types provide consistent error handling
        let test_cases = vec![
            || ContactName::new(""),
            || PhoneNumber::new("123"),
            || SystolicPressure::new(300),
            || BodyTemperature::new(50.0),
            || WeightKg::new(1000.0),
            || Diagnosis::new(&"A".repeat(1500)),
        ];
        
        for test_case in test_cases {
            let result = test_case();
            assert!(result.is_err(), "Should return error for invalid input");
        }
    }
}

// Performance tests to ensure semantic types don't introduce significant overhead
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_semantic_type_creation_performance() {
        let start = Instant::now();
        
        for _ in 0..10000 {
            let _phone = PhoneNumber::new("555-123-4567").unwrap();
            let _pressure = SystolicPressure::new(120).unwrap();
            let _temp = BodyTemperature::new(36.6).unwrap();
        }
        
        let duration = start.elapsed();
        println!("Created 30,000 semantic types in {:?}", duration);
        
        // Should complete quickly (less than 100ms for 30,000 creations)
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_validation_performance() {
        let phone = PhoneNumber::new("555-123-4567").unwrap();
        let start = Instant::now();
        
        for _ in 0..10000 {
            let _formatted = format_phone_number(&phone);
        }
        
        let duration = start.elapsed();
        println!("Formatted 10,000 phone numbers in {:?}", duration);
        
        // Should complete quickly (less than 50ms for 10,000 operations)
        assert!(duration.as_millis() < 50);
    }
}
