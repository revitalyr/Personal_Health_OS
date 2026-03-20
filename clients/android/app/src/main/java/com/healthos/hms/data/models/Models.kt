package com.healthos.hms.data.models

import com.google.gson.annotations.SerializedName

data class LoginRequest(
    val email: String,
    val password: String
)

data class AuthResponse(
    val token: String,
    val refreshToken: String,
    val expiresIn: Long,
    val user: User
)

data class User(
    val id: String,
    val email: String,
    val name: String,
    val role: String
)

data class LicenceValidationRequest(
    val licenceKey: String,
    val hardwareFingerprint: String,
    val clientId: String
)

data class LicenceValidationResponse(
    val licenceId: String,
    val isValid: Boolean,
    val expiryDate: String,
    val tier: String,
    val maxUsers: Int,
    val currentUsers: Int,
    val features: List<String>,
    val validationMessage: String,
    val nextCheck: String
)

data class LicenceStatusResponse(
    val isValid: Boolean,
    val tier: String,
    val features: List<String>,
    val expiryDate: String
)

data class Patient(
    val id: String,
    val patientId: String,
    val name: String,
    val dateOfBirth: String,
    val gender: String,
    val bloodType: String? = null,
    val phone: String? = null,
    val email: String? = null,
    val address: String? = null,
    val emergencyContact: EmergencyContact? = null,
    val status: String,
    val admissionDate: String? = null,
    val dischargeDate: String? = null,
    val createdAt: String,
    val updatedAt: String
)

data class EmergencyContact(
    val name: String,
    val relationship: String,
    val phone: String
)

data class CreatePatientRequest(
    val patientId: String,
    val name: String,
    val dateOfBirth: String,
    val gender: String,
    val bloodType: String? = null,
    val phone: String? = null,
    val email: String? = null,
    val address: String? = null,
    val emergencyContact: EmergencyContact? = null
)

data class UpdatePatientRequest(
    val name: String? = null,
    val phone: String? = null,
    val email: String? = null,
    val address: String? = null,
    val emergencyContact: EmergencyContact? = null
)

data class AdmitPatientRequest(
    val admissionDate: String,
    val facilityId: String,
    val department: String,
    val admittingDoctor: String,
    val diagnosis: String? = null,
    val notes: String? = null
)

data class DischargePatientRequest(
    val dischargeDate: String,
    val dischargeType: String,
    val finalDiagnosis: String,
    val dischargeSummary: String,
    val followUpInstructions: String? = null,
    val prescribedMedications: List<String> = emptyList()
)

data class PatientTimeline(
    val events: List<TimelineEvent>
)

data class TimelineEvent(
    val id: String,
    val type: String,
    val timestamp: String,
    val description: String,
    val data: Map<String, Any>
)

data class Appointment(
    val id: String,
    val patientId: String,
    val patientName: String,
    val doctorId: String,
    val doctorName: String,
    val facilityId: String,
    val facilityName: String,
    val appointmentType: String,
    val status: String,
    val startTime: String,
    val endTime: String,
    val notes: String? = null,
    val createdAt: String,
    val updatedAt: String
)

data class CreateAppointmentRequest(
    val patientId: String,
    val doctorId: String,
    val facilityId: String,
    val appointmentType: String,
    val startTime: String,
    val endTime: String,
    val notes: String? = null
)

data class UpdateAppointmentRequest(
    val status: String? = null,
    val notes: String? = null,
    val startTime: String? = null,
    val endTime: String? = null
)

data class DoctorAvailability(
    val doctorId: String,
    val date: String,
    val timeSlots: List<TimeSlot>
)

data class TimeSlot(
    val startTime: String,
    val endTime: String,
    val isAvailable: Boolean
)

data class Invoice(
    val id: String,
    val invoiceNumber: String,
    val patientId: String,
    val patientName: String,
    val items: List<InvoiceItem>,
    val subtotal: Double,
    val tax: Double,
    val total: Double,
    val status: String,
    val createdAt: String,
    val dueDate: String,
    val payments: List<Payment>
)

data class InvoiceItem(
    val id: String,
    val serviceChargeId: String,
    val description: String,
    val quantity: Int,
    val unitPrice: Double,
    val total: Double
)

data class CreateInvoiceRequest(
    val patientId: String,
    val items: List<CreateInvoiceItemRequest>,
    val dueDate: String,
    val notes: String? = null
)

data class CreateInvoiceItemRequest(
    val serviceChargeId: String,
    val quantity: Int
)

data class Payment(
    val id: String,
    val invoiceId: String,
    val amount: Double,
    val method: String,
    val status: String,
    val transactionId: String? = null,
    val createdAt: String
)

data class CreatePaymentRequest(
    val amount: Double,
    val method: String,
    val transactionId: String? = null
)

data class ServiceCharge(
    val id: String,
    val category: String,
    val description: String,
    val unitPrice: Double,
    val isActive: Boolean
)

data class Facility(
    val id: String,
    val name: String,
    val address: String,
    val phone: String,
    val email: String? = null,
    val isActive: Boolean
)

data class DashboardStats(
    val totalPatients: Int,
    val todayAppointments: Int,
    val pendingInvoices: Int,
    val monthlyRevenue: Double,
    val activeDoctors: Int,
    val occupiedBeds: Int,
    val totalBeds: Int
)

// Response wrappers
data class PatientListResponse(
    val data: List<Patient>,
    val pagination: Pagination
)

data class AppointmentListResponse(
    val data: List<Appointment>,
    val pagination: Pagination
)

data class InvoiceListResponse(
    val data: List<Invoice>,
    val pagination: Pagination
)

data class ServiceChargeListResponse(
    val data: List<ServiceCharge>
)

data class FacilityListResponse(
    val data: List<Facility>
)

data class PatientResponse(
    val data: Patient
)

data class AppointmentResponse(
    val data: Appointment
)

data class InvoiceResponse(
    val data: Invoice
)

data class PaymentResponse(
    val data: Payment
)

data class DashboardStatsResponse(
    val data: DashboardStats
)

data class PatientTimelineResponse(
    val data: PatientTimeline
)

data class DoctorAvailabilityResponse(
    val data: DoctorAvailability
)

data class Pagination(
    val page: Int,
    val limit: Int,
    val total: Int,
    val totalPages: Int
)
