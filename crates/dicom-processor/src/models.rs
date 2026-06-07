use serde::{Deserialize, Serialize};

/// Metadata extracted from a DICOM file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomMetadata {
    pub patient_id: String,
    pub patient_name: String,
    pub study_date: String,
    pub modality: String,
    pub study_description: String,
    pub series_description: String,
    pub body_part_examined: String,
    pub institution_name: String,
    pub physician_name: String,
    pub image_type: Vec<String>,
    pub rows: u32,
    pub columns: u32,
    pub bits_allocated: u32,
    pub bits_stored: u32,
    pub high_bit: u32,
    pub photometric_interpretation: String,
    pub samples_per_pixel: u32,
    pub planar_configuration: Option<u32>,
    pub pixel_representation: u32,
    pub window_center: Option<Vec<f32>>,
    pub window_width: Option<Vec<f32>>,
    pub rescale_intercept: Option<f32>,
    pub rescale_slope: Option<f32>,
}
