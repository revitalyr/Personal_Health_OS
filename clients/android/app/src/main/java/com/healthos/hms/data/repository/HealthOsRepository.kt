package com.healthos.hms.data.repository

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.healthos.hms.data.api.HealthOsApi
import com.healthos.hms.data.models.*
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.security.MessageDigest
import java.util.*

class HealthOsRepository(
    private val context: Context,
    private val api: HealthOsApi
) {
    private val sharedPreferences: SharedPreferences by lazy {
        val masterKey = MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()

        EncryptedSharedPreferences.create(
            context,
            "health_os_prefs",
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )
    }

    private val _authState = MutableStateFlow<AuthState?>(null)
    val authState: Flow<AuthState?> = _authState.asStateFlow()

    private val _licenceState = MutableStateFlow<LicenceState?>(null)
    val licenceState: Flow<LicenceState?> = _licenceState.asStateFlow()

    init {
        loadStoredAuth()
        loadStoredLicence()
    }

    // Authentication
    suspend fun login(email: String, password: String): Result<AuthResponse> {
        return try {
            val response = api.login(LoginRequest(email, password))
            if (response.isSuccessful) {
                response.body()?.let { authResponse ->
                    saveAuthData(authResponse)
                    _authState.value = AuthState.Authenticated(authResponse)
                    Result.success(authResponse)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Login failed: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun logout() {
        sharedPreferences.edit().clear().apply()
        _authState.value = AuthState.LoggedOut
        _licenceState.value = null
    }

    private fun saveAuthData(authResponse: AuthResponse) {
        sharedPreferences.edit().apply {
            putString("token", authResponse.token)
            putString("refresh_token", authResponse.refreshToken)
            putLong("expires_at", System.currentTimeMillis() + authResponse.expiresIn * 1000)
            putString("user_id", authResponse.user.id)
            putString("user_name", authResponse.user.name)
            putString("user_email", authResponse.user.email)
            putString("user_role", authResponse.user.role)
            apply()
        }
    }

    private fun loadStoredAuth() {
        val token = sharedPreferences.getString("token", null)
        val expiresAt = sharedPreferences.getLong("expires_at", 0)
        
        if (token != null && expiresAt > System.currentTimeMillis()) {
            val user = User(
                id = sharedPreferences.getString("user_id", "") ?: "",
                email = sharedPreferences.getString("user_email", "") ?: "",
                name = sharedPreferences.getString("user_name", "") ?: "",
                role = sharedPreferences.getString("user_role", "") ?: ""
            )
            
            val authResponse = AuthResponse(
                token = token,
                refreshToken = sharedPreferences.getString("refresh_token", "") ?: "",
                expiresIn = (expiresAt - System.currentTimeMillis()) / 1000,
                user = user
            )
            
            _authState.value = AuthState.Authenticated(authResponse)
        } else {
            _authState.value = AuthState.LoggedOut
        }
    }

    // Licence Management
    suspend fun validateLicence(licenceKey: String): Result<LicenceValidationResponse> {
        return try {
            val hardwareFingerprint = generateHardwareFingerprint()
            val clientId = UUID.randomUUID().toString()
            
            val response = api.validateLicence(
                LicenceValidationRequest(licenceKey, hardwareFingerprint, clientId)
            )
            
            if (response.isSuccessful) {
                response.body()?.let { validationResponse ->
                    if (validationResponse.isValid) {
                        saveLicenceData(licenceKey, hardwareFingerprint)
                        _licenceState.value = LicenceState.Valid(validationResponse)
                    } else {
                        _licenceState.value = LicenceState.Invalid(validationResponse.validationMessage)
                    }
                    Result.success(validationResponse)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Licence validation failed: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun checkLicenceStatus(): Result<LicenceStatusResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.getLicenceStatus("Bearer $token")
            
            if (response.isSuccessful) {
                response.body()?.let { statusResponse ->
                    if (statusResponse.isValid) {
                        _licenceState.value = LicenceState.Valid(
                            LicenceValidationResponse(
                                licenceId = "",
                                isValid = true,
                                expiryDate = statusResponse.expiryDate,
                                tier = statusResponse.tier,
                                maxUsers = 0,
                                currentUsers = 0,
                                features = statusResponse.features,
                                validationMessage = "Valid",
                                nextCheck = ""
                            )
                        )
                    } else {
                        _licenceState.value = LicenceState.Invalid("Licence expired or invalid")
                    }
                    Result.success(statusResponse)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to check licence status: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    private fun saveLicenceData(licenceKey: String, hardwareFingerprint: String) {
        sharedPreferences.edit().apply {
            putString("licence_key", licenceKey)
            putString("hardware_fingerprint", hardwareFingerprint)
            putLong("licence_validated_at", System.currentTimeMillis())
            apply()
        }
    }

    private fun loadStoredLicence() {
        val licenceKey = sharedPreferences.getString("licence_key", null)
        val hardwareFingerprint = sharedPreferences.getString("hardware_fingerprint", null)
        
        if (licenceKey != null && hardwareFingerprint != null) {
            _licenceState.value = LicenceState.Stored(licenceKey, hardwareFingerprint)
        } else {
            _licenceState.value = LicenceState.NotConfigured
        }
    }

    private fun generateHardwareFingerprint(): String {
        val builder = StringBuilder()
        
        // Collect hardware information
        builder.append("ANDROID_ID:")
        builder.append(android.provider.Settings.Secure.getString(
            context.contentResolver,
            android.provider.Settings.Secure.ANDROID_ID
        ))
        
        builder.append("|BRAND:")
        builder.append(android.os.Build.BRAND)
        
        builder.append("|MODEL:")
        builder.append(android.os.Build.MODEL)
        
        builder.append("|MANUFACTURER:")
        builder.append(android.os.Build.MANUFACTURER)
        
        builder.append("|VERSION:")
        builder.append(android.os.Build.VERSION.RELEASE)
        
        // Create hash
        val digest = MessageDigest.getInstance("SHA-256")
        val hashBytes = digest.digest(builder.toString().toByteArray())
        
        return hashBytes.joinToString("") { "%02x".format(it) }
    }

    private fun getAuthToken(): String? {
        return sharedPreferences.getString("token", null)
    }

    // Patient Management
    suspend fun getPatients(page: Int = 1, limit: Int = 20): Result<PatientListResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.getPatients("Bearer $token", page, limit)
            
            if (response.isSuccessful) {
                response.body()?.let { Result.success(it) }
                    ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get patients: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun createPatient(patient: CreatePatientRequest): Result<PatientResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.createPatient("Bearer $token", patient)
            
            if (response.isSuccessful) {
                response.body()?.let { Result.success(it) }
                    ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to create patient: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    // Appointment Management
    suspend fun getAppointments(date: String, doctorId: String? = null): Result<AppointmentListResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.getAppointments("Bearer $token", date, doctorId)
            
            if (response.isSuccessful) {
                response.body()?.let { Result.success(it) }
                    ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get appointments: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun createAppointment(appointment: CreateAppointmentRequest): Result<AppointmentResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.createAppointment("Bearer $token", appointment)
            
            if (response.isSuccessful) {
                response.body()?.let { Result.success(it) }
                    ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to create appointment: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    // Billing Management
    suspend fun getInvoices(page: Int = 1, limit: Int = 20): Result<InvoiceListResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.getInvoices("Bearer $token", page, limit)
            
            if (response.isSuccessful) {
                response.body()?.let { Result.success(it) }
                    ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get invoices: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun createInvoice(invoice: CreateInvoiceRequest): Result<InvoiceResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.createInvoice("Bearer $token", invoice)
            
            if (response.isSuccessful) {
                response.body()?.let { Result.success(it) }
                    ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to create invoice: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    // Dashboard
    suspend fun getDashboardStats(): Result<DashboardStatsResponse> {
        return try {
            val token = getAuthToken() ?: return Result.failure(Exception("Not authenticated"))
            val response = api.getDashboardStats("Bearer $token")
            
            if (response.isSuccessful) {
                response.body()?.let { Result.success(it) }
                    ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get dashboard stats: ${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

sealed class AuthState {
    object LoggedOut : AuthState()
    data class Authenticated(val authResponse: AuthResponse) : AuthState()
}

sealed class LicenceState {
    object NotConfigured : LicenceState()
    data class Stored(val licenceKey: String, val hardwareFingerprint: String) : LicenceState()
    data class Valid(val validationResponse: LicenceValidationResponse) : LicenceState()
    data class Invalid(val message: String) : LicenceState()
}
