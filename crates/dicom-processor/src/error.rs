use thiserror::Error;

#[derive(Debug, Error)]
pub enum DicomError {
    #[error("Failed to parse DICOM file: {0}")]
    Parse(String),

    #[error("Unsupported DICOM transfer syntax: {0}")]
    UnsupportedSyntax(String),

    #[error("Missing required DICOM tag: {0}")]
    MissingTag(String),
}

pub type Result<T> = std::result::Result<T, DicomError>;
