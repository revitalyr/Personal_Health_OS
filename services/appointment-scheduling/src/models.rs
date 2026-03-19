use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate, NaiveTime, Weekday};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Appointment {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub facility_id: Uuid,
    pub appointment_type: AppointmentType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: AppointmentStatus,
    pub notes: Option<String>,
    pub reminder_sent: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum AppointmentType {
    Consultation,
    FollowUp,
    Procedure,
    Surgery,
    Emergency,
    CheckUp,
    Vaccination,
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    InProgress,
    Completed,
    Cancelled,
    NoShow,
    Rescheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DoctorAvailability {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub day_of_week: i32,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_available: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Facility {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub timezone: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub available: bool,
    pub doctor_id: Uuid,
    pub facility_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentReminder {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub reminder_type: ReminderType,
    pub scheduled_for: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub status: ReminderStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum ReminderType {
    Email,
    SMS,
    Push,
    InApp,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum ReminderStatus {
    Scheduled,
    Sent,
    Failed,
    Cancelled,
}

// Request DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAppointmentRequest {
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub facility_id: Uuid,
    pub appointment_type: AppointmentType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAppointmentRequest {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: Option<AppointmentStatus>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RescheduleAppointmentRequest {
    pub new_start_time: DateTime<Utc>,
    pub new_end_time: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SetDoctorAvailabilityRequest {
    pub doctor_id: Uuid,
    pub day_of_week: i32,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_available: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetAvailableSlotsRequest {
    pub doctor_id: Uuid,
    pub facility_id: Uuid,
    pub date: NaiveDate,
    pub appointment_type: Option<AppointmentType>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateFacilityRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 500))]
    pub address: String,
    #[validate(length(min = 10, max = 20))]
    pub phone: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 50))]
    pub timezone: String,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct AppointmentResponse {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub facility_id: Uuid,
    pub facility_name: Option<String>,
    pub appointment_type: AppointmentType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: AppointmentStatus,
    pub notes: Option<String>,
    pub reminder_sent: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub patient: Option<PatientInfo>,
    pub doctor: Option<DoctorInfo>,
}

#[derive(Debug, Serialize)]
pub struct PatientInfo {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DoctorInfo {
    pub id: Uuid,
    pub name: String,
    pub specialization: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CalendarViewResponse {
    pub date: NaiveDate,
    pub appointments: Vec<AppointmentResponse>,
    pub available_slots: Vec<TimeSlot>,
}

#[derive(Debug, Serialize)]
pub struct DoctorScheduleResponse {
    pub doctor_id: Uuid,
    pub week_start: NaiveDate,
    pub availability: Vec<DoctorAvailability>,
    pub appointments: Vec<AppointmentResponse>,
}

#[derive(Debug, Serialize)]
pub struct AppointmentStatistics {
    pub total_appointments: i64,
    pub scheduled: i64,
    pub completed: i64,
    pub cancelled: i64,
    pub no_shows: i64,
    pub completion_rate: f64,
    pub average_duration_minutes: f64,
}
