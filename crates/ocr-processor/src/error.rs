use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("Failed to load image: {0}")]
    ImageLoad(String),

    #[error("OCR processing failed: {0}")]
    Processing(String),
}

pub type Result<T> = std::result::Result<T, OcrError>;
