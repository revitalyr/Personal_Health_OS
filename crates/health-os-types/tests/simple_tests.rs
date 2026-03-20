// Simple Working Tests for Health OS Semantic Types
// These tests focus on core functionality without complex compilation issues

use health_os_types::*;

#[cfg(test)]
mod simple_tests {
    use super::*;

    #[test]
    fn test_blood_type() {
        let blood_type = BloodType::OPositive;
        assert_eq!(format!("{:?}", blood_type), "OPositive");
    }

    #[test]
    fn test_patient_id() {
        let id1 = new_patient_id();
        let id2 = new_patient_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_contact_name() {
        // Valid name
        let valid = ContactName::new("John Doe");
        assert!(valid.is_ok());
        
        // Invalid empty name
        let invalid = ContactName::new("");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_phone_number() {
        // Valid phone
        let valid = PhoneNumber::new("555-123-4567");
        assert!(valid.is_ok());
        
        // Invalid phone
        let invalid = PhoneNumber::new("123");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_medical_measurements() {
        // Valid measurements
        let systolic = SystolicPressure::new(120);
        let heart_rate = HeartRate::new(72);
        let temp = BodyTemperature::new(36.6);
        
        assert!(systolic.is_ok());
        assert!(heart_rate.is_ok());
        assert!(temp.is_ok());
        
        // Invalid measurements
        let invalid_systolic = SystolicPressure::new(300);
        let invalid_hr = HeartRate::new(10);
        
        assert!(invalid_systolic.is_err());
        assert!(invalid_hr.is_err());
    }

    #[test]
    fn test_blood_pressure_validation() {
        // Valid blood pressure
        let valid = validate_blood_pressure(120, 80);
        assert!(valid.is_ok());
        
        // Invalid (systolic <= diastolic)
        let invalid = validate_blood_pressure(80, 120);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_medical_content() {
        // Valid content
        let diagnosis = Diagnosis::new("Hypertension");
        let treatment = Treatment::new("ACE inhibitors");
        
        assert!(diagnosis.is_ok());
        assert!(treatment.is_ok());
        
        // Invalid content
        let empty = Diagnosis::new("");
        assert!(empty.is_err());
    }

    #[test]
    fn test_enums() {
        // Test all enum variants can be created
        let blood_types = vec![
            BloodType::APositive, BloodType::ANegative,
            BloodType::BPositive, BloodType::BNegative,
            BloodType::ABPositive, BloodType::ABNegative,
            BloodType::OPositive, BloodType::ONegative,
        ];
        
        let statuses = vec![
            PatientStatus::Admitted, PatientStatus::Discharged,
            PatientStatus::Outpatient, PatientStatus::Emergency,
            PatientStatus::Transferred,
        ];
        
        assert_eq!(blood_types.len(), 8);
        assert_eq!(statuses.len(), 5);
    }

    #[test]
    fn test_phone_formatting() {
        if let Ok(phone) = PhoneNumber::new("555-123-4567") {
            let formatted = format_phone_number(&phone);
            assert!(formatted.contains("(") || formatted.contains("-"));
        }
    }
}

#[cfg(test)]
mod patient_tests {
    use super::*;

    #[test]
    fn test_patient_creation() {
        let profile_id = ProfileId::new_v4();
        let patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        assert_eq!(patient.status, PatientStatus::Outpatient);
        assert_eq!(patient.patient_id.as_str(), "P12345");
        assert!(patient.admission_date.is_none());
    }

    #[test]
    fn test_patient_admission() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        // Admit patient
        assert!(patient.admit().is_ok());
        assert_eq!(patient.status, PatientStatus::Admitted);
        assert!(patient.admission_date.is_some());
        
        // Try to admit again
        assert!(patient.admit().is_err());
    }

    #[test]
    fn test_patient_discharge() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        // Try to discharge without admission
        assert!(patient.discharge().is_err());
        
        // Admit then discharge
        assert!(patient.admit().is_ok());
        assert!(patient.discharge().is_ok());
        assert_eq!(patient.status, PatientStatus::Discharged);
    }

    #[test]
    fn test_emergency_contact() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        // Set valid emergency contact
        assert!(patient.set_emergency_contact("John Doe", "555-123-4567").is_ok());
        assert!(patient.emergency_contact_name.is_some());
        
        // Try invalid phone
        assert!(patient.set_emergency_contact("Jane Doe", "123").is_err());
    }

    #[test]
    fn test_insurance() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());
        
        // Set valid insurance
        assert!(patient.set_insurance("Blue Cross", "BC123456789").is_ok());
        assert!(patient.insurance_provider.is_some());
        
        // Try empty provider
        assert!(patient.set_insurance("", "POL123").is_err());
    }
}

#[cfg(test)]
mod encounter_tests {
    use super::*;

    #[test]
    fn test_encounter_creation() {
        let patient_id = PatientId::new_v4();
        let encounter = MedicalEncounter::new(patient_id, EncounterType::Consultation);
        
        assert_eq!(encounter.patient_id, patient_id);
        assert_eq!(encounter.encounter_type, EncounterType::Consultation);
        assert!(encounter.is_active());
    }

    #[test]
    fn test_doctor_assignment() {
        let patient_id = PatientId::new_v4();
        let doctor_id = DoctorId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Emergency);
        
        encounter.assign_doctor(doctor_id);
        assert_eq!(encounter.doctor_id, Some(doctor_id));
    }

    #[test]
    fn test_encounter_ending() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Surgery);
        
        // End encounter
        assert!(encounter.end_encounter().is_ok());
        assert!(!encounter.is_active());
        assert!(encounter.duration().is_some());
        
        // Try to end again
        assert!(encounter.end_encounter().is_err());
    }

    #[test]
    fn test_diagnosis_and_treatment() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Consultation);
        
        // Set diagnosis
        assert!(encounter.set_diagnosis("Hypertension").is_ok());
        assert!(encounter.has_diagnosis());
        
        // Set treatment
        assert!(encounter.set_treatment("Lifestyle changes").is_ok());
        assert!(encounter.has_treatment());
        
        // Try empty diagnosis
        assert!(encounter.set_diagnosis("").is_err());
    }
}

#[cfg(test)]
mod vitals_tests {
    use super::*;

    #[test]
    fn test_vitals_creation() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let vitals = PatientVitals::new(patient_id, staff_id);
        
        assert_eq!(vitals.patient_id, patient_id);
        assert_eq!(vitals.recorded_by, staff_id);
        assert!(vitals.blood_pressure_systolic.is_none());
    }

    #[test]
    fn test_vitals_measurements() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        // Set measurements
        assert!(vitals.set_blood_pressure(120, 80).is_ok());
        assert!(vitals.set_heart_rate(72).is_ok());
        assert!(vitals.set_temperature(36.6).is_ok());
        
        // Verify measurements were set
        assert!(vitals.blood_pressure_systolic.is_some());
        assert!(vitals.heart_rate.is_some());
        assert!(vitals.temperature.is_some());
    }

    #[test]
    fn test_bmi_calculation() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        // Set weight and height
        assert!(vitals.set_weight(70.0).is_ok());
        assert!(vitals.set_height(175.0).is_ok());
        
        // Calculate BMI
        let bmi = vitals.calculate_bmi();
        assert!(bmi.is_some());
        
        let expected_bmi = 70.0 / ((175.0 / 100.0) * (175.0 / 100.0));
        assert!((bmi.unwrap() - expected_bmi).abs() < 0.1);
    }

    #[test]
    fn test_normal_ranges() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        // Normal blood pressure
        assert!(vitals.set_blood_pressure(118, 78).is_ok());
        assert_eq!(vitals.is_blood_pressure_normal(), Some(true));
        
        // High blood pressure
        assert!(vitals.set_blood_pressure(140, 90).is_ok());
        assert_eq!(vitals.is_blood_pressure_normal(), Some(false));
        
        // Normal heart rate
        assert!(vitals.set_heart_rate(72).is_ok());
        assert_eq!(vitals.is_heart_rate_normal(), Some(true));
        
        // Abnormal heart rate
        assert!(vitals.set_heart_rate(120).is_ok());
        assert_eq!(vitals.is_heart_rate_normal(), Some(false));
    }
}

#[cfg(test)]
mod allergy_tests {
    use super::*;

    #[test]
    fn test_allergy_creation() {
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
    }

    #[test]
    fn test_allergy_validation() {
        let patient_id = PatientId::new_v4();
        
        // Valid allergen names
        assert!(PatientAllergy::new(patient_id, "Peanuts", AllergySeverity::Mild).is_ok());
        assert!(PatientAllergy::new(patient_id, "Penicillin", AllergySeverity::Moderate).is_ok());
        
        // Invalid allergen names
        assert!(PatientAllergy::new(patient_id, "", AllergySeverity::Mild).is_err());
        
        let long_name = "A".repeat(200);
        assert!(PatientAllergy::new(patient_id, &long_name, AllergySeverity::Mild).is_err());
    }

    #[test]
    fn test_allergy_severity() {
        let patient_id = PatientId::new_v4();
        
        let mild = PatientAllergy::new(patient_id, "Dust", AllergySeverity::Mild).unwrap();
        assert!(!mild.is_severe());
        assert!(!mild.is_life_threatening());
        
        let severe = PatientAllergy::new(patient_id, "Bee stings", AllergySeverity::Severe).unwrap();
        assert!(severe.is_severe());
        assert!(!severe.is_life_threatening());
        
        let life_threatening = PatientAllergy::new(patient_id, "Peanuts", AllergySeverity::LifeThreatening).unwrap();
        assert!(life_threatening.is_severe());
        assert!(life_threatening.is_life_threatening());
    }

    #[test]
    fn test_allergy_categories() {
        let patient_id = PatientId::new_v4();
        
        let medication = PatientAllergy::new(patient_id, "Penicillin", AllergySeverity::Moderate).unwrap();
        assert_eq!(medication.get_allergy_category(), AllergyCategory::Medication);
        
        let food = PatientAllergy::new(patient_id, "Shellfish", AllergySeverity::Severe).unwrap();
        assert_eq!(food.get_allergy_category(), AllergyCategory::Food);
        
        let environmental = PatientAllergy::new(patient_id, "Pollen", AllergySeverity::Mild).unwrap();
        assert_eq!(environmental.get_allergy_category(), AllergyCategory::Environmental);
    }

    #[test]
    fn test_emergency_actions() {
        let patient_id = PatientId::new_v4();
        
        let mild = PatientAllergy::new(patient_id, "Dust", AllergySeverity::Mild).unwrap();
        let mild_actions = mild.get_emergency_actions();
        assert!(mild_actions.contains(&"Monitor for symptoms"));
        
        let severe = PatientAllergy::new(patient_id, "Bee stings", AllergySeverity::Severe).unwrap();
        let severe_actions = severe.get_emergency_actions();
        assert!(severe_actions.contains(&"Call emergency services"));
        
        let life_threatening = PatientAllergy::new(patient_id, "Peanuts", AllergySeverity::LifeThreatening).unwrap();
        let lt_actions = life_threatening.get_emergency_actions();
        assert!(lt_actions.contains(&"EMERGENCY: Call 911 immediately"));
    }
}

#[cfg(test)]
mod medication_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_medication_creation() {
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
        assert_eq!(med.route, AdministrationRoute::Oral);
        assert!(med.is_active());
    }

    #[test]
    fn test_medication_validation() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        // Valid medications
        assert!(PatientMedication::new(
            patient_id,
            "Metformin",
            "500mg",
            "Twice daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).is_ok());

        // Invalid medication names
        assert!(PatientMedication::new(
            patient_id,
            "",
            "10mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).is_err());

        // Invalid dosage
        assert!(PatientMedication::new(
            patient_id,
            "Aspirin",
            "",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).is_err());
    }

    #[test]
    fn test_medication_lifecycle() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut medication = PatientMedication::new(
            patient_id,
            "Amoxicillin",
            "500mg",
            "Three times daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();

        // Initial state
        assert!(medication.is_active());
        assert!(!medication.is_completed());

        // Complete medication
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 22).unwrap();
        assert!(medication.end_medication(end_date).is_ok());
        assert!(medication.is_completed());
        assert!(!medication.is_active());

        // Try to complete again
        assert!(medication.end_medication(end_date).is_err());
    }

    #[test]
    fn test_medication_hold_resume() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut medication = PatientMedication::new(
            patient_id,
            "Warfarin",
            "5mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();

        // Put on hold
        assert!(medication.put_on_hold().is_ok());
        assert!(medication.is_on_hold());
        assert!(!medication.is_active());

        // Resume from hold
        assert!(medication.resume_medication().is_ok());
        assert!(medication.is_active());
        assert!(!medication.is_on_hold());
    }

    #[test]
    fn test_medication_routes() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

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

        let topical = PatientMedication::new(
            patient_id,
            "Hydrocortisone",
            "1%",
            "Twice daily",
            AdministrationRoute::Topical,
            start_date,
            prescriber_id,
        ).unwrap();
        assert_eq!(topical.route_display(), "Topical");
    }

    #[test]
    fn test_monitoring_requirements() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Oral medication - no monitoring required
        let oral = PatientMedication::new(
            patient_id,
            "Aspirin",
            "81mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        assert!(!oral.requires_monitoring());

        // IV medication - monitoring required
        let iv = PatientMedication::new(
            patient_id,
            "Morphine",
            "2mg",
            "As needed",
            AdministrationRoute::Intravenous,
            start_date,
            prescriber_id,
        ).unwrap();
        assert!(iv.requires_monitoring());

        // IM medication - monitoring required
        let im = PatientMedication::new(
            patient_id,
            "Vitamin B12",
            "1000mcg",
            "Monthly",
            AdministrationRoute::Intramuscular,
            start_date,
            prescriber_id,
        ).unwrap();
        assert!(im.requires_monitoring());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

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
