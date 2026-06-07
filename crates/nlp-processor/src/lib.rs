pub mod models;
pub mod error;

pub use error::NlpError;
pub use models::MedicalEntity;

/// Medical entity extractor — identifies clinical concepts in free text.
pub struct MedicalEntityExtractor;

impl MedicalEntityExtractor {
    /// Extract medical entities from text using keyword and regex patterns.
    pub fn extract_entities(text: &str) -> Result<Vec<MedicalEntity>, NlpError> {
        let mut entities = Vec::new();

        // Simple keyword-based entity extraction
        let medication_keywords = [
            "амоксициллин", "ибупрофен", "парацетамол", "аспирин", "антибиотик",
        ];
        let symptom_keywords = [
            "головная боль", "температура", "кашель", "боль в горле", "насморк",
        ];
        let diagnosis_keywords = [
            "грипп", "простуда", "бронхит", "пневмония", "ангина",
        ];

        for (pos, word) in text.split_whitespace().enumerate() {
            let lower_word = word.to_lowercase();

            if medication_keywords.contains(&lower_word.as_str()) {
                entities.push(MedicalEntity {
                    text: word.to_string(),
                    entity_type: "medication".to_string(),
                    confidence: 0.8,
                    start_pos: pos,
                    end_pos: pos + word.len(),
                    normalized_value: Some(word.to_lowercase()),
                });
            } else if symptom_keywords.iter().any(|kw| lower_word.contains(kw)) {
                entities.push(MedicalEntity {
                    text: word.to_string(),
                    entity_type: "symptom".to_string(),
                    confidence: 0.7,
                    start_pos: pos,
                    end_pos: pos + word.len(),
                    normalized_value: None,
                });
            } else if diagnosis_keywords.contains(&lower_word.as_str()) {
                entities.push(MedicalEntity {
                    text: word.to_string(),
                    entity_type: "diagnosis".to_string(),
                    confidence: 0.9,
                    start_pos: pos,
                    end_pos: pos + word.len(),
                    normalized_value: Some(word.to_lowercase()),
                });
            }
        }

        // Regex-based extraction for structured patterns
        Self::extract_with_regex(text, &mut entities)?;

        Ok(entities)
    }

    fn extract_with_regex(text: &str, entities: &mut Vec<MedicalEntity>) -> Result<(), NlpError> {
        use regex::Regex;

        // Extract dates
        let date_re = Regex::new(r"\b\d{2}\.\d{2}\.\d{4}\b")?;
        for m in date_re.find_iter(text) {
            entities.push(MedicalEntity {
                text: m.as_str().to_string(),
                entity_type: "date".to_string(),
                confidence: 0.9,
                start_pos: m.start(),
                end_pos: m.end(),
                normalized_value: Some(m.as_str().to_string()),
            });
        }

        // Extract lab values (e.g., "120/80", "5.2 mmol/l")
        let lab_re = Regex::new(r"\b\d+\.?\d*\s*(?:мм рт\. ст\.|ммоль/л|нг/мл|ед/л)\b")?;
        for m in lab_re.find_iter(text) {
            entities.push(MedicalEntity {
                text: m.as_str().to_string(),
                entity_type: "lab_value".to_string(),
                confidence: 0.8,
                start_pos: m.start(),
                end_pos: m.end(),
                normalized_value: Some(m.as_str().to_string()),
            });
        }

        // Extract medications with dosage
        let med_re = Regex::new(r"\b(\d+)\s*(мг|мл|таблетка|капсула)\s+(амоксициллин|ибупрофен|парацетамол)\b")?;
        for m in med_re.find_iter(text) {
            entities.push(MedicalEntity {
                text: m.as_str().to_string(),
                entity_type: "medication".to_string(),
                confidence: 0.85,
                start_pos: m.start(),
                end_pos: m.end(),
                normalized_value: Some(m.as_str().to_lowercase()),
            });
        }

        Ok(())
    }
}
