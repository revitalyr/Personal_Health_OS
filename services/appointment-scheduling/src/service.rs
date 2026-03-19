use crate::models::*;
use crate::error::AppointmentError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate, NaiveTime, Duration, Weekday};
use serde_json::json;

pub struct AppointmentService {
    db: PgPool,
}

impl AppointmentService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // Appointment Management
    pub async fn create_appointment(&self, request: CreateAppointmentRequest) -> Result<Appointment, AppointmentError> {
        // Check for conflicts
        if self.has_conflict(request.doctor_id, request.start_time, request.end_time).await? {
            return Err(AppointmentError::TimeSlotConflict);
        }

        let appointment = sqlx::query_as!(
            Appointment,
            r#"
            INSERT INTO appointments (
                patient_id, doctor_id, facility_id, appointment_type,
                start_time, end_time, status, notes, reminder_sent,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, patient_id, doctor_id, facility_id,
                      appointment_type as "appointment_type: AppointmentType",
                      start_time, end_time, status as "status: AppointmentStatus",
                      notes, reminder_sent, created_at, updated_at
            "#,
            request.patient_id,
            request.doctor_id,
            request.facility_id,
            request.appointment_type,
            request.start_time,
            request.end_time,
            AppointmentStatus::Scheduled,
            request.notes,
            false,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Schedule reminders
        self.schedule_appointment_reminders(appointment.id).await?;

        // Log appointment creation
        self.log_appointment_event(appointment.id, "appointment_created", json!({
            "patient_id": request.patient_id,
            "doctor_id": request.doctor_id,
            "appointment_type": request.appointment_type
        })).await?;

        Ok(appointment)
    }

    pub async fn get_appointment(&self, appointment_id: Uuid) -> Result<Appointment, AppointmentError> {
        sqlx::query_as!(
            Appointment,
            r#"
            SELECT id, patient_id, doctor_id, facility_id,
                   appointment_type as "appointment_type: AppointmentType",
                   start_time, end_time, status as "status: AppointmentStatus",
                   notes, reminder_sent, created_at, updated_at
            FROM appointments
            WHERE id = $1
            "#,
            appointment_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppointmentError::AppointmentNotFound)
    }

    pub async fn update_appointment(&self, appointment_id: Uuid, request: UpdateAppointmentRequest) -> Result<Appointment, AppointmentError> {
        let appointment = self.get_appointment(appointment_id).await?;

        // Check for conflicts if time is being updated
        if let (Some(start_time), Some(end_time)) = (&request.start_time, &request.end_time) {
            if self.has_conflict_excluding(appointment.doctor_id, *start_time, *end_time, appointment_id).await? {
                return Err(AppointmentError::TimeSlotConflict);
            }
        }

        let updated_appointment = sqlx::query_as!(
            Appointment,
            r#"
            UPDATE appointments
            SET start_time = COALESCE($2, start_time),
                end_time = COALESCE($3, end_time),
                status = COALESCE($4, status),
                notes = COALESCE($5, notes),
                updated_at = $6
            WHERE id = $1
            RETURNING id, patient_id, doctor_id, facility_id,
                      appointment_type as "appointment_type: AppointmentType",
                      start_time, end_time, status as "status: AppointmentStatus",
                      notes, reminder_sent, created_at, updated_at
            "#,
            appointment_id,
            request.start_time,
            request.end_time,
            request.status,
            request.notes,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        // Log appointment update
        self.log_appointment_event(appointment_id, "appointment_updated", json!({
            "updated_fields": request
        })).await?;

        Ok(updated_appointment)
    }

    pub async fn cancel_appointment(&self, appointment_id: Uuid, reason: Option<String>) -> Result<Appointment, AppointmentError> {
        let appointment = self.update_appointment(
            appointment_id,
            UpdateAppointmentRequest {
                status: Some(AppointmentStatus::Cancelled),
                notes: reason,
                ..Default::default()
            }
        ).await?;

        // Cancel scheduled reminders
        self.cancel_appointment_reminders(appointment_id).await?;

        // Log cancellation
        self.log_appointment_event(appointment_id, "appointment_cancelled", json!({
            "reason": reason
        })).await?;

        Ok(appointment)
    }

    pub async fn reschedule_appointment(&self, appointment_id: Uuid, request: RescheduleAppointmentRequest) -> Result<Appointment, AppointmentError> {
        let appointment = self.get_appointment(appointment_id).await?;

        // Check for conflicts at new time
        if self.has_conflict_excluding(appointment.doctor_id, request.new_start_time, request.new_end_time, appointment_id).await? {
            return Err(AppointmentError::TimeSlotConflict);
        }

        let updated_appointment = self.update_appointment(
            appointment_id,
            UpdateAppointmentRequest {
                start_time: Some(request.new_start_time),
                end_time: Some(request.new_end_time),
                status: Some(AppointmentStatus::Rescheduled),
                ..Default::default()
            }
        ).await?;

        // Reschedule reminders
        self.reschedule_appointment_reminders(appointment_id).await?;

        // Log reschedule
        self.log_appointment_event(appointment_id, "appointment_rescheduled", json!({
            "old_start_time": appointment.start_time,
            "new_start_time": request.new_start_time,
            "reason": request.reason
        })).await?;

        Ok(updated_appointment)
    }

    pub async fn list_appointments(&self, filters: AppointmentFilters) -> Result<Vec<Appointment>, AppointmentError> {
        let mut query = "
            SELECT id, patient_id, doctor_id, facility_id,
                   appointment_type as \"appointment_type: AppointmentType\",
                   start_time, end_time, status as \"status: AppointmentStatus\",
                   notes, reminder_sent, created_at, updated_at
            FROM appointments
            WHERE 1=1
        ".to_string();

        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(patient_id) = filters.patient_id {
            query.push_str(&format!(" AND patient_id = ${}", param_index));
            params.push(patient_id);
            param_index += 1;
        }

        if let Some(doctor_id) = filters.doctor_id {
            query.push_str(&format!(" AND doctor_id = ${}", param_index));
            params.push(doctor_id);
            param_index += 1;
        }

        if let Some(facility_id) = filters.facility_id {
            query.push_str(&format!(" AND facility_id = ${}", param_index));
            params.push(facility_id);
            param_index += 1;
        }

        if let Some(status) = filters.status {
            query.push_str(&format!(" AND status = ${}", param_index));
            params.push(status);
            param_index += 1;
        }

        if let Some(start_date) = filters.start_date {
            query.push_str(&format!(" AND start_time >= ${}", param_index));
            params.push(start_date);
            param_index += 1;
        }

        if let Some(end_date) = filters.end_date {
            query.push_str(&format!(" AND start_time <= ${}", param_index));
            params.push(end_date);
            param_index += 1;
        }

        query.push_str(" ORDER BY start_time ASC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT ${}", param_index));
            params.push(limit);
            param_index += 1;
        }

        let mut query_builder = sqlx::query_as::<_, Appointment>(&query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }

        query_builder.fetch_all(&self.db).await.map_err(AppointmentError::Database)
    }

    // Availability Management
    pub async fn set_doctor_availability(&self, request: SetDoctorAvailabilityRequest) -> Result<DoctorAvailability, AppointmentError> {
        let availability = sqlx::query_as!(
            DoctorAvailability,
            r#"
            INSERT INTO doctor_availability (
                doctor_id, day_of_week, start_time, end_time, is_available, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (doctor_id, day_of_week)
            DO UPDATE SET
                start_time = EXCLUDED.start_time,
                end_time = EXCLUDED.end_time,
                is_available = EXCLUDED.is_available
            RETURNING id, doctor_id, day_of_week, start_time, end_time, is_available, created_at
            "#,
            request.doctor_id,
            request.day_of_week,
            request.start_time,
            request.end_time,
            request.is_available,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        Ok(availability)
    }

    pub async fn get_doctor_availability(&self, doctor_id: Uuid) -> Result<Vec<DoctorAvailability>, AppointmentError> {
        sqlx::query_as!(
            DoctorAvailability,
            r#"
            SELECT id, doctor_id, day_of_week, start_time, end_time, is_available, created_at
            FROM doctor_availability
            WHERE doctor_id = $1
            ORDER BY day_of_week
            "#,
            doctor_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(AppointmentError::Database)
    }

    pub async fn get_available_slots(&self, request: GetAvailableSlotsRequest) -> Result<Vec<TimeSlot>, AppointmentError> {
        let doctor_availability = self.get_doctor_availability(request.doctor_id).await?;
        
        let day_of_week = request.date.weekday().num_days_from_monday() as i32;
        let availability = doctor_availability.iter()
            .find(|avail| avail.day_of_week == day_of_week && avail.is_available);

        if availability.is_none() {
            return Ok(Vec::new());
        }

        let availability = availability.unwrap();
        let date_utc = request.date.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Get existing appointments for the day
        let start_of_day = date_utc;
        let end_of_day = date_utc + Duration::days(1);

        let existing_appointments = sqlx::query!(
            r#"
            SELECT start_time, end_time
            FROM appointments
            WHERE doctor_id = $1
              AND facility_id = $2
              AND start_time >= $3
              AND start_time < $4
              AND status NOT IN ('cancelled', 'no_show')
            ORDER BY start_time
            "#,
            request.doctor_id,
            request.facility_id,
            start_of_day,
            end_of_day
        )
        .fetch_all(&self.db)
        .await
        .map_err(AppointmentError::Database)?;

        // Generate time slots
        let mut slots = Vec::new();
        let slot_duration = Duration::minutes(30); // 30-minute slots

        let availability_start = date_utc.with_time(availability.start_time).unwrap().and_utc();
        let availability_end = date_utc.with_time(availability.end_time).unwrap().and_utc();

        let mut current_time = availability_start;
        
        while current_time + slot_duration <= availability_end {
            let slot_end = current_time + slot_duration;
            
            // Check if slot conflicts with existing appointments
            let is_available = !existing_appointments.iter().any(|apt| {
                (current_time < apt.end_time && slot_end > apt.start_time)
            });

            slots.push(TimeSlot {
                start_time: current_time,
                end_time: slot_end,
                available: is_available,
                doctor_id: request.doctor_id,
                facility_id: request.facility_id,
            });

            current_time = slot_end;
        }

        Ok(slots)
    }

    // Facility Management
    pub async fn create_facility(&self, request: CreateFacilityRequest) -> Result<Facility, AppointmentError> {
        let facility = sqlx::query_as!(
            Facility,
            r#"
            INSERT INTO facilities (name, address, phone, email, timezone, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, address, phone, email, timezone, is_active, created_at, updated_at
            "#,
            request.name,
            request.address,
            request.phone,
            request.email,
            request.timezone,
            true,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        Ok(facility)
    }

    pub async fn get_facilities(&self) -> Result<Vec<Facility>, AppointmentError> {
        sqlx::query_as!(
            Facility,
            r#"
            SELECT id, name, address, phone, email, timezone, is_active, created_at, updated_at
            FROM facilities
            WHERE is_active = true
            ORDER BY name
            "#
        )
        .fetch_all(&self.db)
        .await
        .map_err(AppointmentError::Database)
    }

    // Statistics
    pub async fn get_appointment_statistics(&self, filters: AppointmentFilters) -> Result<AppointmentStatistics, AppointmentError> {
        let appointments = self.list_appointments(filters).await?;
        
        let total_appointments = appointments.len() as i64;
        let scheduled = appointments.iter().filter(|a| matches!(a.status, AppointmentStatus::Scheduled)).count() as i64;
        let completed = appointments.iter().filter(|a| matches!(a.status, AppointmentStatus::Completed)).count() as i64;
        let cancelled = appointments.iter().filter(|a| matches!(a.status, AppointmentStatus::Cancelled)).count() as i64;
        let no_shows = appointments.iter().filter(|a| matches!(a.status, AppointmentStatus::NoShow)).count() as i64;

        let completion_rate = if total_appointments > 0 {
            completed as f64 / total_appointments as f64 * 100.0
        } else {
            0.0
        };

        let average_duration = if completed > 0 {
            let total_duration: i64 = appointments.iter()
                .filter(|a| matches!(a.status, AppointmentStatus::Completed))
                .map(|a| (a.end_time - a.start_time).num_minutes())
                .sum();
            total_duration as f64 / completed as f64
        } else {
            0.0
        };

        Ok(AppointmentStatistics {
            total_appointments,
            scheduled,
            completed,
            cancelled,
            no_shows,
            completion_rate,
            average_duration_minutes: average_duration,
        })
    }

    // Helper methods
    async fn has_conflict(&self, doctor_id: Uuid, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<bool, AppointmentError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM appointments
            WHERE doctor_id = $1
              AND status NOT IN ('cancelled', 'no_show')
              AND (
                  (start_time < $2 AND end_time > $2) OR
                  (start_time < $3 AND end_time > $3) OR
                  (start_time >= $2 AND end_time <= $3)
              )
            "#,
            doctor_id,
            start_time,
            end_time
        )
        .fetch_one(&self.db)
        .await
        .map_err(AppointmentError::Database)?;

        Ok(count > 0)
    }

    async fn has_conflict_excluding(&self, doctor_id: Uuid, start_time: DateTime<Utc>, end_time: DateTime<Utc>, exclude_id: Uuid) -> Result<bool, AppointmentError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM appointments
            WHERE doctor_id = $1
              AND id != $2
              AND status NOT IN ('cancelled', 'no_show')
              AND (
                  (start_time < $3 AND end_time > $3) OR
                  (start_time < $4 AND end_time > $4) OR
                  (start_time >= $3 AND end_time <= $4)
              )
            "#,
            doctor_id,
            exclude_id,
            start_time,
            end_time
        )
        .fetch_one(&self.db)
        .await
        .map_err(AppointmentError::Database)?;

        Ok(count > 0)
    }

    async fn schedule_appointment_reminders(&self, appointment_id: Uuid) -> Result<(), AppointmentError> {
        // Schedule reminders for 24 hours before and 2 hours before
        let appointment = self.get_appointment(appointment_id).await?;
        
        let reminder_times = vec![
            appointment.start_time - Duration::hours(24),
            appointment.start_time - Duration::hours(2),
        ];

        for scheduled_for in reminder_times {
            if scheduled_for > Utc::now() {
                sqlx::query!(
                    r#"
                    INSERT INTO appointment_reminders (appointment_id, reminder_type, scheduled_for, status, created_at)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                    appointment_id,
                    ReminderType::Email,
                    scheduled_for,
                    ReminderStatus::Scheduled,
                    Utc::now()
                )
                .execute(&self.db)
                .await
                .map_err(AppointmentError::Database)?;
            }
        }

        Ok(())
    }

    async fn cancel_appointment_reminders(&self, appointment_id: Uuid) -> Result<(), AppointmentError> {
        sqlx::query!(
            r#"
            UPDATE appointment_reminders
            SET status = 'cancelled'
            WHERE appointment_id = $1 AND status = 'scheduled'
            "#,
            appointment_id
        )
        .execute(&self.db)
        .await
        .map_err(AppointmentError::Database)?;

        Ok(())
    }

    async fn reschedule_appointment_reminders(&self, appointment_id: Uuid) -> Result<(), AppointmentError> {
        self.cancel_appointment_reminders(appointment_id).await?;
        self.schedule_appointment_reminders(appointment_id).await?;
        Ok(())
    }

    async fn log_appointment_event(&self, appointment_id: Uuid, event_type: &str, metadata: serde_json::Value) -> Result<(), AppointmentError> {
        sqlx::query!(
            r#"
            INSERT INTO appointment_events (appointment_id, event_type, metadata, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            appointment_id,
            event_type,
            metadata,
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(AppointmentError::Database)?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AppointmentFilters {
    pub patient_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub facility_id: Option<Uuid>,
    pub status: Option<AppointmentStatus>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}
