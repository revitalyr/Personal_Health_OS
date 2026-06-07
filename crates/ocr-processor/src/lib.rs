pub mod models;
pub mod error;

pub use error::OcrError;
pub use models::OcrResult;

/// OCR processor — extracts text from image data.
///
/// Currently a stub that returns mock text. Replace with Tesseract
/// or another OCR engine for production use.
pub struct OcrProcessor;

impl OcrProcessor {
    /// Perform OCR on raw image data.
    /// Returns extracted text with confidence score.
    pub fn perform_ocr(data: &[u8]) -> Result<OcrResult, OcrError> {
        let img = image::load_from_memory(data)
            .map_err(|e| OcrError::ImageLoad(e.to_string()))?;

        // Convert to grayscale for better OCR
        let _gray_img = img.to_luma8();

        // TODO: integrate real OCR engine (tesseract, etc.)
        // Mock OCR result for demo
        Ok(OcrResult {
            text: "Sample OCR text for demo purposes".to_string(),
            confidence: 0.95,
            language_detected: "eng".to_string(),
            processing_time_ms: 100,
        })
    }
}
