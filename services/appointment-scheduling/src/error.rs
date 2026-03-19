use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppointmentError {
    #[error("Appointment not found")]
    AppointmentNotFound,
    
    #[error("Time slot conflict")]
    TimeSlotConflict,
    
    #[error("Invalid appointment data: {0}")]
    InvalidData(String),
    
    #[error("Doctor not available at requested time")]
    DoctorNotAvailable,
    
    #[error("Facility not found")]
    FacilityNotFound,
    
    #[error("Patient not found")]
    PatientNotFound,
    
    #[error("Doctor not found")]
    DoctorNotFound,
    
    #[error("Cannot cancel completed appointment")]
    CannotCancelCompleted,
    
    #[error("Cannot reschedule past appointment")]
    CannotReschedulePast,
    
    #[error("Invalid time range")]
    InvalidTimeRange,
    
    #[error("Reminder scheduling failed")]
    ReminderSchedulingFailed,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Service temporarily unavailable")]
    ServiceUnavailable,
}
