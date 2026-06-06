// Comprehensive Test Suite for Health OS Semantic Types
// This file contains extensive tests demonstrating the benefits and functionality
// of semantic types for healthcare data management

use health_os_types::*;
use chrono::{Utc, NaiveDate};

#[cfg(test)]
mod semantic_types_tests {
    use super::*;

    #[test]
    fn test_patient_id_creation() {
        let patient_id = new_patient_id();
        assert!(patient_id.to_string().len() > 0);
    }

    #[test]
    fn test_blood_type_variants() {
        let blood_types = vec![
            BloodType::APositive,
            BloodType::ANegative,
            BloodType::BPositive,
            BloodType::BNegative,
            BloodType::ABPositive,
            BloodType::ABNegative,
            BloodType::OPositive,
            BloodType::ONegative,
        ];

        for blood_type in blood_types {
            // Verify all blood types can be created and serialized
            let serialized = serde_json::to_string(&blood_type).unwrap();
            let deserialized: BloodType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(blood_type, deserialized);
        }
    }

    #[test]
    fn test_contact_name_validation() {
        // Valid contact names
        assert!(ContactName::new("John Doe").is_ok());
        assert!(ContactName::new("Dr. Jane Smith").is_ok());
        assert!(ContactName::new("José María González").is_ok());

        // Invalid contact names
        assert!(ContactName::new("").is_err());
        assert!(ContactName::new("   ").is_err());
        
        // Too long name
        let long_name = "A".repeat(200);
        assert!(ContactName::new(&long_name).is_err());
    }

    #[test]
    fn test_phone_number_validation() {
        // Valid phone numbers
        assert!(PhoneNumber::new("555-123-4567").is_ok());
        assert!(PhoneNumber::new("(555) 123-4567").is_ok());
        assert!(PhoneNumber::new("5551234567").is_ok());
        assert!(PhoneNumber::new("+1-555-123-4567").is_ok());

        // Invalid phone numbers
        assert!(PhoneNumber::new("123").is_err());
        assert!(PhoneNumber::new("").is_err());
        assert!(PhoneNumber::new("abc-def-ghij").is_err());
        assert!(PhoneNumber::new("555-123-45678").is_err()); // Too long
    }

    #[test]
    fn test_phone_number_formatting() {
        let phone = PhoneNumber::new("5551234567").unwrap();
        let formatted = format_phone_number(&phone);
        
        assert!(formatted.contains("("));
        assert!(formatted.contains(")"));
        assert!(formatted.contains("-"));
        assert_eq!(formatted, "(555) 123-4567");
    }

    #[test]
    fn test_medical_measurements() {
        // Blood pressure
        let systolic = SystolicPressure::new(120).unwrap();
        let diastolic = DiastolicPressure::new(80).unwrap();
        assert_eq!(systolic.value(), 120);
        assert_eq!(diastolic.value(), 80);

        // Invalid blood pressure
        assert!(SystolicPressure::new(300).is_err());
        assert!(DiastolicPressure::new(200).is_err());

        // Heart rate
        let heart_rate = HeartRate::new(72).unwrap();
        assert_eq!(heart_rate.value(), 72);
        assert!(HeartRate::new(10).is_err());
        assert!(HeartRate::new(300).is_err());

        // Temperature
        let temp = BodyTemperature::new(36.6).unwrap();
        assert_eq!(temp.value(), 36.6);
        assert!(BodyTemperature::new(50.0).is_err());
        assert!(BodyTemperature::new(20.0).is_err());

        // Weight and height
        let weight = WeightKg::new(70.0).unwrap();
        let height = HeightCm::new(175.0).unwrap();
        assert_eq!(weight.value(), 70.0);
        assert_eq!(height.value(), 175.0);

        // Oxygen saturation
        let oxygen = OxygenSaturation::new(98.0).unwrap();
        assert_eq!(oxygen.value(), 98.0);
        assert!(OxygenSaturation::new(150.0).is_err());
    }

    #[test]
    fn test_blood_pressure_validation() {
        // Valid blood pressure
        let (systolic, diastolic) = validate_blood_pressure(120, 80).unwrap();
        assert_eq!(systolic.value(), 120);
        assert_eq!(diastolic.value(), 80);

        // Invalid blood pressure (systolic <= diastolic)
        assert!(validate_blood_pressure(80, 120).is_err());

        // Invalid range
        assert!(validate_blood_pressure(300, 80).is_err());
        assert!(validate_blood_pressure(120, 200).is_err());
    }

    #[test]
    fn test_medical_content_types() {
        // Diagnosis
        let diagnosis = Diagnosis::new("Hypertension").unwrap();
        assert_eq!(diagnosis.as_str(), "Hypertension");
        assert!(Diagnosis::new("").is_err());
        assert!(Diagnosis::new(&"A".repeat(1500)).is_err());

        // Treatment
        let treatment = Treatment::new("Prescribed ACE inhibitors").unwrap();
        assert_eq!(treatment.as_str(), "Prescribed ACE inhibitors");
        assert!(Treatment::new(&"A".repeat(3000)).is_err());

        // Notes
        let notes = Notes::new("Patient responded well to treatment").unwrap();
        assert_eq!(notes.as_str(), "Patient responded well to treatment");
        assert!(Notes::new(&"A".repeat(6000)).is_err());
    }

    #[test]
    fn test_temporal_types() {
        let created = CreatedAt::now();
        let updated = UpdatedAt::now();
        
        assert!(created.value() <= updated.value());
        assert!(created.value() <= Utc::now());
        assert!(updated.value() <= Utc::now());

        let start_date = StartDate::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let end_date = EndDate::new(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap());
        
        assert!(start_date.value() < end_date.value());
    }
}

#[cfg(test)]
mod hospital_patient_tests {
    use super::*;

    #[test]
    fn test_patient_lifecycle() {
        let profile_id = ProfileId::new_v4();
        let patient_code = "P12345".to_string();
        
        let mut patient = HospitalPatient::new(profile_id, patient_code);
        
        // Initial state
        assert_eq!(patient.status, PatientStatus::Outpatient);
        assert!(patient.admission_date.is_none());
        assert!(patient.discharge_date.is_none());

        // Admit patient
        assert!(patient.admit().is_ok());
        assert_eq!(patient.status, PatientStatus::Admitted);
        assert!(patient.admission_date.is_some());
        assert!(patient.discharge_date.is_none());

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
    fn test_emergency_contact_management() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());

        // Set valid emergency contact
        assert!(patient.set_emergency_contact("John Doe", "555-123-4567").is_ok());
        assert!(patient.emergency_contact_name.is_some());
        assert!(patient.emergency_contact_phone.is_some());

        let display = patient.get_emergency_contact_display();
        assert!(display.is_some());
        let display_str = display.unwrap();
        assert!(display_str.contains("John Doe"));
        assert!(display_str.contains("(555)"));

        // Try invalid contact
        assert!(patient.set_emergency_contact("", "123").is_err());
    }

    #[test]
    fn test_insurance_management() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());

        // Set valid insurance
        assert!(patient.set_insurance("Blue Cross Blue Shield", "BC123456789").is_ok());
        assert!(patient.insurance_provider.is_some());
        assert!(patient.insurance_policy_number.is_some());

        // Try invalid insurance
        assert!(patient.set_insurance("", "POL123").is_err());
        assert!(patient.set_insurance("Aetna", "").is_err());
    }

    #[test]
    fn test_length_of_stay() {
        let profile_id = ProfileId::new_v4();
        let mut patient = HospitalPatient::new(profile_id, "P12345".to_string());

        // No stay yet
        assert!(patient.length_of_stay_days().is_none());

        // Admit patient
        assert!(patient.admit().is_ok());
        let stay_days = patient.length_of_stay_days();
        assert!(stay_days.is_some());
        assert!(stay_days.unwrap() >= 0);

        // Discharge after 7 days
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(patient.discharge().is_ok());
        let final_stay = patient.length_of_stay_days();
        assert!(final_stay.is_some());
        assert!(final_stay.unwrap() >= 0);
    }
}

#[cfg(test)]
mod medical_encounter_tests {
    use super::*;

    #[test]
    fn test_encounter_creation() {
        let patient_id = PatientId::new_v4();
        let encounter = MedicalEncounter::new(patient_id, EncounterType::Consultation);

        assert_eq!(encounter.patient_id, patient_id);
        assert_eq!(encounter.encounter_type, EncounterType::Consultation);
        assert!(encounter.is_active());
        assert!(encounter.doctor_id.is_none());
        assert!(encounter.diagnosis.is_none());
        assert!(encounter.treatment.is_none());
    }

    #[test]
    fn test_encounter_lifecycle() {
        let patient_id = PatientId::new_v4();
        let doctor_id = DoctorId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Emergency);

        // Assign doctor
        encounter.assign_doctor(doctor_id);
        assert_eq!(encounter.doctor_id, Some(doctor_id));

        // Set diagnosis and treatment
        assert!(encounter.set_diagnosis("Acute appendicitis").is_ok());
        assert!(encounter.set_treatment("Appendectomy").is_ok());
        assert!(encounter.has_diagnosis());
        assert!(encounter.has_treatment());

        // End encounter
        assert!(encounter.end_encounter().is_ok());
        assert!(!encounter.is_active());
        assert!(encounter.duration().is_some());

        // Try to end again
        assert!(encounter.end_encounter().is_err());
    }

    #[test]
    fn test_encounter_types() {
        let patient_id = PatientId::new_v4();
        let encounter_types = vec![
            EncounterType::Consultation,
            EncounterType::Emergency,
            EncounterType::Surgery,
            EncounterType::FollowUp,
            EncounterType::Diagnostic,
            EncounterType::Therapy,
        ];

        for encounter_type in encounter_types {
            let encounter = MedicalEncounter::new(patient_id, encounter_type.clone());
            assert_eq!(encounter.encounter_type, encounter_type);
            
            let display_name = encounter.encounter_type_display();
            assert!(!display_name.is_empty());
        }
    }

    #[test]
    fn test_encounter_duration() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Surgery);

        // No duration initially
        assert!(encounter.duration().is_none());
        assert!(encounter.duration_minutes().is_none());

        // End encounter and check duration
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(encounter.end_encounter().is_ok());
        
        let duration = encounter.duration();
        assert!(duration.is_some());
        assert!(duration.unwrap().num_milliseconds() >= 100);
        
        let minutes = encounter.duration_minutes();
        assert!(minutes.is_some());
        assert!(minutes.unwrap() >= 0);
    }
}

#[cfg(test)]
mod patient_vitals_tests {
    use super::*;

    #[test]
    fn test_vitals_creation() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let vitals = PatientVitals::new(patient_id, staff_id);

        assert_eq!(vitals.patient_id, patient_id);
        assert_eq!(vitals.recorded_by, staff_id);
        assert!(vitals.blood_pressure_systolic.is_none());
        assert!(vitals.heart_rate.is_none());
        assert!(vitals.temperature.is_none());
    }

    #[test]
    fn test_vitals_measurements() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);

        // Set all measurements
        assert!(vitals.set_blood_pressure(120, 80).is_ok());
        assert!(vitals.set_heart_rate(72).is_ok());
        assert!(vitals.set_temperature(36.6).is_ok());
        assert!(vitals.set_weight(70.0).is_ok());
        assert!(vitals.set_height(175.0).is_ok());
        assert!(vitals.set_oxygen_saturation(98.0).is_ok());

        // Verify measurements
        assert_eq!(vitals.blood_pressure_systolic.unwrap().value(), 120);
        assert_eq!(vitals.blood_pressure_diastolic.unwrap().value(), 80);
        assert_eq!(vitals.heart_rate.unwrap().value(), 72);
        assert_eq!(vitals.temperature.unwrap().value(), 36.6);
        assert_eq!(vitals.weight.unwrap().value(), 70.0);
        assert_eq!(vitals.height.unwrap().value(), 175.0);
        assert_eq!(vitals.oxygen_saturation.unwrap().value(), 98.0);
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

        // Check BMI category
        let category = vitals.get_bmi_category();
        assert_eq!(category, Some("Normal weight"));
    }

    #[test]
    fn test_vitals_normal_ranges() {
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

        // Normal temperature
        assert!(vitals.set_temperature(36.6).is_ok());
        assert_eq!(vitals.is_temperature_normal(), Some(true));

        // Abnormal temperature
        assert!(vitals.set_temperature(38.5).is_ok());
        assert_eq!(vitals.is_temperature_normal(), Some(false));

        // Normal oxygen saturation
        assert!(vitals.set_oxygen_saturation(98.0).is_ok());
        assert_eq!(vitals.is_oxygen_saturation_normal(), Some(true));

        // Low oxygen saturation
        assert!(vitals.set_oxygen_saturation(92.0).is_ok());
        assert_eq!(vitals.is_oxygen_saturation_normal(), Some(false));
    }

    #[test]
    fn test_vitals_summary() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);

        // Set all measurements
        assert!(vitals.set_blood_pressure(120, 80).is_ok());
        assert!(vitals.set_heart_rate(72).is_ok());
        assert!(vitals.set_temperature(36.6).is_ok());
        assert!(vitals.set_weight(70.0).is_ok());
        assert!(vitals.set_height(175.0).is_ok());
        assert!(vitals.set_oxygen_saturation(98.0).is_ok());

        // Get summary
        let summary = vitals.get_measurements_summary();

        assert_eq!(summary.blood_pressure, Some("120/80".to_string()));
        assert_eq!(summary.heart_rate, Some("72".to_string()));
        assert_eq!(summary.temperature, Some("36.6°C".to_string()));
        assert_eq!(summary.weight, Some("70.0 kg".to_string()));
        assert_eq!(summary.height, Some("175.0 cm".to_string()));
        assert_eq!(summary.oxygen_saturation, Some("98.0%".to_string()));
        assert!(summary.bmi.is_some());
        assert_eq!(summary.bmi_category, Some("Normal weight".to_string()));
    }
}

#[cfg(test)]
mod patient_allergy_tests {
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
        assert!(allergy.reaction.is_none());
        assert!(allergy.notes.is_none());
    }

    #[test]
    fn test_allergy_validation() {
        let patient_id = PatientId::new_v4();

        // Valid allergen names
        assert!(PatientAllergy::new(patient_id, "Peanuts", AllergySeverity::Mild).is_ok());
        assert!(PatientAllergy::new(patient_id, "Penicillin", AllergySeverity::Moderate).is_ok());
        assert!(PatientAllergy::new(patient_id, "Latex", AllergySeverity::Severe).is_ok());

        // Invalid allergen names
        assert!(PatientAllergy::new(patient_id, "", AllergySeverity::Mild).is_err());
        let long_name = "A".repeat(200);
        assert!(PatientAllergy::new(patient_id, &long_name, AllergySeverity::Mild).is_err());
    }

    #[test]
    fn test_allergy_severity_classification() {
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

        let material = PatientAllergy::new(patient_id, "Latex", AllergySeverity::Moderate).unwrap();
        assert_eq!(material.get_allergy_category(), AllergyCategory::Material);
    }

    #[test]
    fn test_emergency_actions() {
        let patient_id = PatientId::new_v4();
        
        let mild = PatientAllergy::new(patient_id, "Dust", AllergySeverity::Mild).unwrap();
        let mild_actions = mild.get_emergency_actions();
        assert!(mild_actions.iter().any(|s| s == "Monitor for symptoms"));
        assert!(!mild_actions.iter().any(|s| s == "Call emergency services"));

        let severe = PatientAllergy::new(patient_id, "Bee stings", AllergySeverity::Severe).unwrap();
        let severe_actions = severe.get_emergency_actions();
        assert!(severe_actions.iter().any(|s| s == "Call emergency services"));
        assert!(severe_actions.iter().any(|s| s == "Administer epinephrine if available"));

        let life_threatening = PatientAllergy::new(patient_id, "Peanuts", AllergySeverity::LifeThreatening).unwrap();
        let lt_actions = life_threatening.get_emergency_actions();
        assert!(lt_actions.iter().any(|s| s == "EMERGENCY: Call 911 immediately"));
        assert!(lt_actions.iter().any(|s| s == "Administer epinephrine immediately"));
    }

    #[test]
    fn test_allergy_summary() {
        let patient_id = PatientId::new_v4();
        let mut allergy = PatientAllergy::new(
            patient_id,
            "Latex",
            AllergySeverity::Moderate,
        ).unwrap();

        assert!(allergy.set_reaction("Skin rash, itching").is_ok());
        assert!(allergy.set_notes("Avoid latex products, use alternative materials").is_ok());

        let summary = allergy.create_summary();

        assert_eq!(summary.allergen, "Latex");
        assert_eq!(summary.severity, "Moderate");
        assert_eq!(summary.category, "Material");
        assert_eq!(summary.reaction, Some("Skin rash, itching".to_string()));
        assert_eq!(summary.notes, Some("Avoid latex products, use alternative materials".to_string()));
        assert!(!summary.is_severe);
        assert!(!summary.is_life_threatening);
        assert!(summary.emergency_actions.iter().any(|s| s == "Immediate medical attention recommended"));
    }
}

#[cfg(test)]
mod patient_medication_tests {
    use super::*;

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
        assert_eq!(med.frequency.as_str(), "Once daily");
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
        assert!(!medication.is_discontinued());
        assert!(!medication.is_on_hold());

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

        // Try to put on hold again
        assert!(medication.put_on_hold().is_ok());
        assert!(medication.is_on_hold());

        // Try to resume when not on hold
        assert!(medication.resume_medication().is_ok());
        assert!(medication.is_active());
    }

    #[test]
    fn test_medication_discontinuation() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut medication = PatientMedication::new(
            patient_id,
            "Metformin",
            "500mg",
            "Twice daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();

        // Discontinue medication
        assert!(medication.discontinue_medication("Side effects").is_ok());
        assert!(medication.is_discontinued());
        assert!(!medication.is_active());

        // Try to discontinue again
        assert!(medication.discontinue_medication("Side effects").is_err());

        // Try to end completed medication
        let end_date = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        assert!(medication.end_medication(end_date).is_err());
    }

    #[test]
    fn test_medication_routes() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let routes = vec![
            AdministrationRoute::Oral,
            AdministrationRoute::Intravenous,
            AdministrationRoute::Intramuscular,
            AdministrationRoute::Subcutaneous,
            AdministrationRoute::Topical,
            AdministrationRoute::Inhalation,
        ];

        for route in routes {
            let medication = PatientMedication::new(
                patient_id,
                "Test Medication",
                "10mg",
                "Once daily",
                route.clone(),
                start_date,
                prescriber_id,
            ).unwrap();

            let display = medication.route_display();
            assert!(!display.is_empty());
        }
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

    #[test]
    fn test_medication_summary() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let medication = PatientMedication::new(
            patient_id,
            "Metformin",
            "500mg",
            "Twice daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();

        let summary = medication.create_summary();

        assert_eq!(summary.medication_name, "Metformin");
        assert_eq!(summary.dosage, "500mg");
        assert_eq!(summary.frequency, "Twice daily");
        assert_eq!(summary.route, "Oral");
        assert_eq!(summary.status, "Active");
        assert_eq!(summary.start_date, start_date);
        assert!(summary.end_date.is_none());
        assert!(!summary.requires_monitoring);
        assert!(summary.is_active);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_create_patient_request_flow() {
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
    fn test_semantic_type_benefits_demonstration() {
        // This test demonstrates the practical benefits of semantic types
        
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
        assert!(formatted_phone.contains("(") && formatted_phone.contains(")"));
        
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
        // Test that all semantic types can be serialized and deserialized
        let patient_id = PatientId::new_v4();
        let blood_type = BloodType::ABNegative;
        let phone = PhoneNumber::new("555-123-4567").unwrap();
        
        // Serialize
        let patient_serialized = serde_json::to_string(&patient_id).unwrap();
        let blood_type_serialized = serde_json::to_string(&blood_type).unwrap();
        let phone_serialized = serde_json::to_string(&phone).unwrap();
        
        // Deserialize
        let patient_deserialized: PatientId = serde_json::from_str(&patient_serialized).unwrap();
        let blood_type_deserialized: BloodType = serde_json::from_str(&blood_type_serialized).unwrap();
        let phone_deserialized: PhoneNumber = serde_json::from_str(&phone_serialized).unwrap();
        
        // Verify roundtrip
        assert_eq!(patient_id, patient_deserialized);
        assert_eq!(blood_type, blood_type_deserialized);
        assert_eq!(phone.as_str(), phone_deserialized.as_str());
    }

    #[test]
    fn test_error_handling_consistency() {
        // Test that all semantic types provide consistent error handling
        let test_cases: Vec<Box<dyn Fn() -> Result<(), &'static str>>> = vec![
            Box::new(|| ContactName::new("").map(|_| ())),
            Box::new(|| PhoneNumber::new("123").map(|_| ())),
            Box::new(|| SystolicPressure::new(300).map(|_| ())),
            Box::new(|| BodyTemperature::new(50.0).map(|_| ())),
            Box::new(|| WeightKg::new(1000.0).map(|_| ())),
            Box::new(|| Diagnosis::new(&"A".repeat(1500)).map(|_| ())),
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
