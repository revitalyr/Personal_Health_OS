use thiserror::Error;

#[derive(Debug, Error)]
pub enum NlpError {
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Entity extraction failed: {0}")]
    Extraction(String),
}

pub type Result<T> = std::result::Result<T, NlpError>;
