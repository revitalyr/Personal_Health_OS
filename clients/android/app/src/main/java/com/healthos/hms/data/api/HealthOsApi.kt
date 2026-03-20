package com.healthos.hms.data.api

import retrofit2.Response
import retrofit2.http.*
import com.healthos.hms.data.models.*

interface HealthOsApi {
    
    // Authentication
    @POST("/auth/login")
    suspend fun login(@Body request: LoginRequest): Response<AuthResponse>
    
    @POST("/auth/refresh")
    suspend fun refreshToken(@Body request: RefreshTokenRequest): Response<AuthResponse>
    
    // Licence Management
    @POST("/api/v1/validate")
    suspend fun validateLicence(@Body request: LicenceValidationRequest): Response<LicenceValidationResponse>
    
    @GET("/api/v1/licence/status")
    suspend fun getLicenceStatus(@Header("Authorization") token: String): Response<LicenceStatusResponse>
    
    // Patient Management
    @GET("/api/v1/patients")
    suspend fun getPatients(
        @Header("Authorization") token: String,
        @Query("page") page: Int,
        @Query("limit") limit: Int
    ): Response<PatientListResponse>
    
    @POST("/api/v1/patients")
    suspend fun createPatient(
        @Header("Authorization") token: String,
        @Body patient: CreatePatientRequest
    ): Response<PatientResponse>
    
    @GET("/api/v1/patients/{id}")
    suspend fun getPatient(
        @Header("Authorization") token: String,
        @Path("id") id: String
    ): Response<PatientResponse>
    
    @PUT("/api/v1/patients/{id}")
    suspend fun updatePatient(
        @Header("Authorization") token: String,
        @Path("id") id: String,
        @Body patient: UpdatePatientRequest
    ): Response<PatientResponse>
    
    @POST("/api/v1/patients/{id}/admit")
    suspend fun admitPatient(
        @Header("Authorization") token: String,
        @Path("id") id: String,
        @Body request: AdmitPatientRequest
    ): Response<PatientResponse>
    
    @POST("/api/v1/patients/{id}/discharge")
    suspend fun dischargePatient(
        @Header("Authorization") token: String,
        @Path("id") id: String,
        @Body request: DischargePatientRequest
    ): Response<PatientResponse>
    
    @GET("/api/v1/patients/{id}/timeline")
    suspend fun getPatientTimeline(
        @Header("Authorization") token: String,
        @Path("id") id: String,
        @Query("limit") limit: Int = 50
    ): Response<PatientTimelineResponse>
    
    // Appointment Management
    @GET("/api/v1/appointments")
    suspend fun getAppointments(
        @Header("Authorization") token: String,
        @Query("date") date: String,
        @Query("doctor_id") doctorId: String? = null
    ): Response<AppointmentListResponse>
    
    @POST("/api/v1/appointments")
    suspend fun createAppointment(
        @Header("Authorization") token: String,
        @Body appointment: CreateAppointmentRequest
    ): Response<AppointmentResponse>
    
    @GET("/api/v1/appointments/{id}")
    suspend fun getAppointment(
        @Header("Authorization") token: String,
        @Path("id") id: String
    ): Response<AppointmentResponse>
    
    @PUT("/api/v1/appointments/{id}")
    suspend fun updateAppointment(
        @Header("Authorization") token: String,
        @Path("id") id: String,
        @Body appointment: UpdateAppointmentRequest
    ): Response<AppointmentResponse>
    
    @POST("/api/v1/appointments/{id}/cancel")
    suspend fun cancelAppointment(
        @Header("Authorization") token: String,
        @Path("id") id: String
    ): Response<AppointmentResponse>
    
    @GET("/api/v1/doctors/{id}/availability")
    suspend fun getDoctorAvailability(
        @Header("Authorization") token: String,
        @Path("id") id: String,
        @Query("start_date") startDate: String,
        @Query("end_date") endDate: String
    ): Response<DoctorAvailabilityResponse>
    
    // Billing & Invoicing
    @GET("/api/v1/invoices")
    suspend fun getInvoices(
        @Header("Authorization") token: String,
        @Query("page") page: Int,
        @Query("limit") limit: Int
    ): Response<InvoiceListResponse>
    
    @POST("/api/v1/invoices")
    suspend fun createInvoice(
        @Header("Authorization") token: String,
        @Body invoice: CreateInvoiceRequest
    ): Response<InvoiceResponse>
    
    @GET("/api/v1/invoices/{id}")
    suspend fun getInvoice(
        @Header("Authorization") token: String,
        @Path("id") id: String
    ): Response<InvoiceResponse>
    
    @POST("/api/v1/invoices/{id}/payments")
    suspend fun addPayment(
        @Header("Authorization") token: String,
        @Path("id") id: String,
        @Body payment: CreatePaymentRequest
    ): Response<PaymentResponse>
    
    @GET("/api/v1/service-charges")
    suspend fun getServiceCharges(
        @Header("Authorization") token: String
    ): Response<ServiceChargeListResponse>
    
    // Dashboard
    @GET("/api/v1/dashboard/stats")
    suspend fun getDashboardStats(
        @Header("Authorization") token: String
    ): Response<DashboardStatsResponse>
    
    // Facilities
    @GET("/api/v1/facilities")
    suspend fun getFacilities(
        @Header("Authorization") token: String
    ): Response<FacilityListResponse>
}
