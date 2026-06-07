use serde::{Deserialize, Serialize};

/// Result of OCR processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// Extracted text content
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Detected language
    pub language_detected: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}
