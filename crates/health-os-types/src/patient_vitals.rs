// Refactored Patient Vitals using semantic types
// This demonstrates how semantic types improve readability and type safety

use crate::semantic_types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Patient vitals with semantic type aliases for medical measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientVitals {
    /// Unique identifier for the vitals record
    pub id: VitalsId,
    
    /// Reference to the patient
    pub patient_id: PatientId,
    
    /// Systolic blood pressure reading
    pub blood_pressure_systolic: Option<SystolicPressure>,
    
    /// Diastolic blood pressure reading
    pub blood_pressure_diastolic: Option<DiastolicPressure>,
    
    /// Heart rate measurement
    pub heart_rate: Option<HeartRate>,
    
    /// Body temperature in Celsius
    pub temperature: Option<BodyTemperature>,
    
    /// Patient weight in kilograms
    pub weight: Option<WeightKg>,
    
    /// Patient height in centimeters
    pub height: Option<HeightCm>,
    
    /// Oxygen saturation percentage
    pub oxygen_saturation: Option<OxygenSaturation>,
    
    /// When these vitals were recorded
    pub recorded_at: RecordedAt,
    
    /// Staff member who recorded the vitals
    pub recorded_by: StaffId,
}

impl PatientVitals {
    /// Create a new vitals record
    pub fn new(patient_id: PatientId, recorded_by: StaffId) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_id,
            blood_pressure_systolic: None,
            blood_pressure_diastolic: None,
            heart_rate: None,
            temperature: None,
            weight: None,
            height: None,
            oxygen_saturation: None,
            recorded_at: RecordedAt::now(),
            recorded_by,
        }
    }
    
    /// Set blood pressure reading
    pub fn set_blood_pressure(
        &mut self,
        systolic: i32,
        diastolic: i32,
    ) -> Result<(), String> {
        let (validated_systolic, validated_diastolic) = validate_blood_pressure(systolic, diastolic)?;
        
        self.blood_pressure_systolic = Some(validated_systolic);
        self.blood_pressure_diastolic = Some(validated_diastolic);
        self.recorded_at = RecordedAt::now();
        
        Ok(())
    }
    
    /// Set heart rate
    pub fn set_heart_rate(&mut self, rate: i32) -> Result<(), String> {
        let validated_rate = HeartRate::new(rate)
            .map_err(|e| e.to_string())?;
        
        self.heart_rate = Some(validated_rate);
        self.recorded_at = RecordedAt::now();
        
        Ok(())
    }
    
    /// Set body temperature
    pub fn set_temperature(&mut self, temp: f32) -> Result<(), String> {
        let validated_temp = BodyTemperature::new(temp)
            .map_err(|e| e.to_string())?;
        
        self.temperature = Some(validated_temp);
        self.recorded_at = RecordedAt::now();
        
        Ok(())
    }
    
    /// Set weight
    pub fn set_weight(&mut self, weight_kg: f32) -> Result<(), String> {
        let validated_weight = WeightKg::new(weight_kg)
            .map_err(|e| e.to_string())?;
        
        self.weight = Some(validated_weight);
        self.recorded_at = RecordedAt::now();
        
        Ok(())
    }
    
    /// Set height
    pub fn set_height(&mut self, height_cm: f32) -> Result<(), String> {
        let validated_height = HeightCm::new(height_cm)
            .map_err(|e| e.to_string())?;
        
        self.height = Some(validated_height);
        self.recorded_at = RecordedAt::now();
        
        Ok(())
    }
    
    /// Set oxygen saturation
    pub fn set_oxygen_saturation(&mut self, saturation: f32) -> Result<(), String> {
        let validated_saturation = OxygenSaturation::new(saturation)
            .map_err(|e| e.to_string())?;
        
        self.oxygen_saturation = Some(validated_saturation);
        self.recorded_at = RecordedAt::now();
        
        Ok(())
    }
    
    /// Get blood pressure as formatted string
    pub fn get_blood_pressure_display(&self) -> Option<String> {
        match (self.blood_pressure_systolic.clone(), self.blood_pressure_diastolic.clone()) {
            (Some(systolic), Some(diastolic)) => {
                Some(format!("{}/{}", systolic.value(), diastolic.value()))
            }
            _ => None,
        }
    }
    
    /// Calculate BMI if weight and height are available
    pub fn calculate_bmi(&self) -> Option<f32> {
        match (self.weight.clone(), self.height.clone()) {
            (Some(weight), Some(height)) => {
                let weight_kg = weight.value();
                let height_m = height.value() / 100.0;
                Some(weight_kg / (height_m * height_m))
            }
            _ => None,
        }
    }
    
    /// Get BMI category
    pub fn get_bmi_category(&self) -> Option<&'static str> {
        self.calculate_bmi().map(|bmi| {
            match bmi {
                bmi if bmi < 18.5 => "Underweight",
                bmi if bmi < 25.0 => "Normal weight",
                bmi if bmi < 30.0 => "Overweight",
                _ => "Obese",
            }
        })
    }
    
    /// Check if blood pressure is normal
    pub fn is_blood_pressure_normal(&self) -> Option<bool> {
        match (self.blood_pressure_systolic.clone(), self.blood_pressure_diastolic.clone()) {
            (Some(systolic), Some(diastolic)) => {
                let sys = systolic.value();
                let dia = diastolic.value();
                Some(sys <= 120 && dia <= 80)
            }
            _ => None,
        }
    }
    
    /// Check if heart rate is normal (for adults)
    pub fn is_heart_rate_normal(&self) -> Option<bool> {
        self.heart_rate.clone().map(|rate| {
            let rate_val = rate.value();
            rate_val >= 60 && rate_val <= 100
        })
    }
    
    /// Check if temperature is normal
    pub fn is_temperature_normal(&self) -> Option<bool> {
        self.temperature.clone().map(|temp| {
            let temp_val = temp.value();
            temp_val >= 36.1 && temp_val <= 37.2
        })
    }
    
    /// Check if oxygen saturation is normal
    pub fn is_oxygen_saturation_normal(&self) -> Option<bool> {
        self.oxygen_saturation.clone().map(|sat| sat.value() >= 95.0)
    }
    
    /// Get all available measurements as a summary
    pub fn get_measurements_summary(&self) -> VitalsSummary {
        VitalsSummary {
            blood_pressure: self.get_blood_pressure_display(),
            heart_rate: self.heart_rate.clone().map(|hr| hr.value().to_string()),
            temperature: self.temperature.clone().map(|t| format!("{:.1}°C", t.value())),
            weight: self.weight.clone().map(|w| format!("{:.1} kg", w.value())),
            height: self.height.clone().map(|h| format!("{:.1} cm", h.value())),
            oxygen_saturation: self.oxygen_saturation.clone().map(|o| format!("{:.1}%", o.value())),
            bmi: self.calculate_bmi().map(|bmi| format!("{:.1}", bmi)),
            bmi_category: self.get_bmi_category().map(|s| s.to_string()),
        }
    }
}

/// Summary of patient vitals for display purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalsSummary {
    pub blood_pressure: Option<String>,
    pub heart_rate: Option<String>,
    pub temperature: Option<String>,
    pub weight: Option<String>,
    pub height: Option<String>,
    pub oxygen_saturation: Option<String>,
    pub bmi: Option<String>,
    pub bmi_category: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vitals_creation() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        
        let vitals = PatientVitals::new(patient_id, staff_id);
        
        assert!(vitals.blood_pressure_systolic.is_none());
        assert!(vitals.heart_rate.is_none());
        assert!(vitals.temperature.is_none());
    }
    
    #[test]
    fn test_blood_pressure_setting() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        // Set valid blood pressure
        assert!(vitals.set_blood_pressure(120, 80).is_ok());
        assert!(vitals.blood_pressure_systolic.is_some());
        assert!(vitals.blood_pressure_diastolic.is_some());
        
        // Try invalid blood pressure
        assert!(vitals.set_blood_pressure(80, 120).is_err()); // systolic < diastolic
        assert!(vitals.set_blood_pressure(300, 80).is_err()); // systolic too high
    }
    
    #[test]
    fn test_heart_rate_setting() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        // Set valid heart rate
        assert!(vitals.set_heart_rate(72).is_ok());
        assert_eq!(vitals.heart_rate.clone().unwrap().value(), 72);
        
        // Try invalid heart rate
        assert!(vitals.set_heart_rate(10).is_err()); // too low
        assert!(vitals.set_heart_rate(300).is_err()); // too high
    }
    
    #[test]
    fn test_temperature_setting() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        // Set valid temperature
        assert!(vitals.set_temperature(36.6).is_ok());
        assert_eq!(vitals.temperature.clone().unwrap().value(), 36.6);
        
        // Try invalid temperature
        assert!(vitals.set_temperature(50.0).is_err()); // too high
        assert!(vitals.set_temperature(20.0).is_err()); // too low
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
    fn test_blood_pressure_normal() {
        let patient_id = PatientId::new_v4();
        let staff_id = StaffId::new_v4();
        let mut vitals = PatientVitals::new(patient_id, staff_id);
        
        // Normal blood pressure
        assert!(vitals.set_blood_pressure(118, 78).is_ok());
        assert_eq!(vitals.is_blood_pressure_normal(), Some(true));
        
        // High blood pressure
        assert!(vitals.set_blood_pressure(140, 90).is_ok());
        assert_eq!(vitals.is_blood_pressure_normal(), Some(false));
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
    }
}
