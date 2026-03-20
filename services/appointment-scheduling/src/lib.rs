//! Appointment scheduling service
//! 
//! This service provides functionality for managing medical appointments,
//! including scheduling, rescheduling, cancellation, and notifications.

pub mod error;
pub mod models;
pub mod service;

pub use error::*;
pub use models::*;
pub use service::*;
