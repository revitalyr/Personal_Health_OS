import Foundation
import Alamofire
import CryptoSwift
import KeychainAccess
import LocalAuthentication

// MARK: - API Client
class HealthOSAPIClient {
    static let shared = HealthOSAPIClient()
    
    private let baseURL: String
    private let session: Session
    
    private init() {
        self.baseURL = "https://api.healthos.app"
        
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = 30
        configuration.timeoutIntervalForResource = 60
        
        self.session = Session(configuration: configuration)
    }
    
    // MARK: - Authentication
    func login(email: String, password: String) async throws -> AuthResponse {
        let url = "\(baseURL)/auth/login"
        let parameters = LoginRequest(email: email, password: password)
        
        return try await session.request(
            url,
            method: .post,
            parameters: parameters,
            encoder: JSONParameterEncoder.default
        )
        .serializingDecodable(AuthResponse.self)
        .value
    }
    
    func refreshToken(refreshToken: String) async throws -> AuthResponse {
        let url = "\(baseURL)/auth/refresh"
        let parameters = RefreshTokenRequest(refreshToken: refreshToken)
        
        return try await session.request(
            url,
            method: .post,
            parameters: parameters,
            encoder: JSONParameterEncoder.default
        )
        .serializingDecodable(AuthResponse.self)
        .value
    }
    
    // MARK: - Licence Management
    func validateLicence(licenceKey: String, hardwareFingerprint: String) async throws -> LicenceValidationResponse {
        let url = "\(baseURL)/api/v1/validate"
        let clientId = UUID().uuidString
        let parameters = LicenceValidationRequest(
            licenceKey: licenceKey,
            hardwareFingerprint: hardwareFingerprint,
            clientId: clientId
        )
        
        return try await session.request(
            url,
            method: .post,
            parameters: parameters,
            encoder: JSONParameterEncoder.default
        )
        .serializingDecodable(LicenceValidationResponse.self)
        .value
    }
    
    func getLicenceStatus(token: String) async throws -> LicenceStatusResponse {
        let url = "\(baseURL)/api/v1/licence/status"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .get,
            headers: headers
        )
        .serializingDecodable(LicenceStatusResponse.self)
        .value
    }
    
    // MARK: - Patient Management
    func getPatients(token: String, page: Int = 1, limit: Int = 20) async throws -> PatientListResponse {
        let url = "\(baseURL)/api/v1/patients"
        let parameters = ["page": page, "limit": limit]
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .get,
            parameters: parameters,
            headers: headers
        )
        .serializingDecodable(PatientListResponse.self)
        .value
    }
    
    func createPatient(token: String, patient: CreatePatientRequest) async throws -> PatientResponse {
        let url = "\(baseURL)/api/v1/patients"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .post,
            parameters: patient,
            encoder: JSONParameterEncoder.default,
            headers: headers
        )
        .serializingDecodable(PatientResponse.self)
        .value
    }
    
    func getPatient(token: String, id: String) async throws -> PatientResponse {
        let url = "\(baseURL)/api/v1/patients/\(id)"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .get,
            headers: headers
        )
        .serializingDecodable(PatientResponse.self)
        .value
    }
    
    func updatePatient(token: String, id: String, patient: UpdatePatientRequest) async throws -> PatientResponse {
        let url = "\(baseURL)/api/v1/patients/\(id)"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .put,
            parameters: patient,
            encoder: JSONParameterEncoder.default,
            headers: headers
        )
        .serializingDecodable(PatientResponse.self)
        .value
    }
    
    func admitPatient(token: String, id: String, admission: AdmitPatientRequest) async throws -> PatientResponse {
        let url = "\(baseURL)/api/v1/patients/\(id)/admit"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .post,
            parameters: admission,
            encoder: JSONParameterEncoder.default,
            headers: headers
        )
        .serializingDecodable(PatientResponse.self)
        .value
    }
    
    func dischargePatient(token: String, id: String, discharge: DischargePatientRequest) async throws -> PatientResponse {
        let url = "\(baseURL)/api/v1/patients/\(id)/discharge"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .post,
            parameters: discharge,
            encoder: JSONParameterEncoder.default,
            headers: headers
        )
        .serializingDecodable(PatientResponse.self)
        .value
    }
    
    func getPatientTimeline(token: String, id: String, limit: Int = 50) async throws -> PatientTimelineResponse {
        let url = "\(baseURL)/api/v1/patients/\(id)/timeline"
        let parameters = ["limit": limit]
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .get,
            parameters: parameters,
            headers: headers
        )
        .serializingDecodable(PatientTimelineResponse.self)
        .value
    }
    
    // MARK: - Appointment Management
    func getAppointments(token: String, date: String, doctorId: String? = nil) async throws -> AppointmentListResponse {
        let url = "\(baseURL)/api/v1/appointments"
        var parameters: [String: Any] = ["date": date]
        
        if let doctorId = doctorId {
            parameters["doctor_id"] = doctorId
        }
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .get,
            parameters: parameters,
            headers: headers
        )
        .serializingDecodable(AppointmentListResponse.self)
        .value
    }
    
    func createAppointment(token: String, appointment: CreateAppointmentRequest) async throws -> AppointmentResponse {
        let url = "\(baseURL)/api/v1/appointments"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .post,
            parameters: appointment,
            encoder: JSONParameterEncoder.default,
            headers: headers
        )
        .serializingDecodable(AppointmentResponse.self)
        .value
    }
    
    // MARK: - Billing & Invoicing
    func getInvoices(token: String, page: Int = 1, limit: Int = 20) async throws -> InvoiceListResponse {
        let url = "\(baseURL)/api/v1/invoices"
        let parameters = ["page": page, "limit": limit]
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .get,
            parameters: parameters,
            headers: headers
        )
        .serializingDecodable(InvoiceListResponse.self)
        .value
    }
    
    func createInvoice(token: String, invoice: CreateInvoiceRequest) async throws -> InvoiceResponse {
        let url = "\(baseURL)/api/v1/invoices"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .post,
            parameters: invoice,
            encoder: JSONParameterEncoder.default,
            headers: headers
        )
        .serializingDecodable(InvoiceResponse.self)
        .value
    }
    
    func getDashboardStats(token: String) async throws -> DashboardStatsResponse {
        let url = "\(baseURL)/api/v1/dashboard/stats"
        
        let headers: HTTPHeaders = [
            "Authorization": "Bearer \(token)"
        ]
        
        return try await session.request(
            url,
            method: .get,
            headers: headers
        )
        .serializingDecodable(DashboardStatsResponse.self)
        .value
    }
}

// MARK: - Hardware Fingerprint
extension HealthOSAPIClient {
    func generateHardwareFingerprint() -> String {
        var components: [String] = []
        
        // Device model
        if let model = UIDevice.current.model {
            components.append("MODEL:\(model)")
        }
        
        // System name
        if let systemName = UIDevice.current.systemName {
            components.append("SYSTEM:\(systemName)")
        }
        
        // System version
        if let systemVersion = UIDevice.current.systemVersion {
            components.append("VERSION:\(systemVersion)")
        }
        
        // Device identifier
        if let identifierForVendor = UIDevice.current.identifierForVendor {
            components.append("VENDOR:\(identifierForVendor.uuidString)")
        }
        
        // Create SHA-256 hash
        let fingerprint = components.joined(separator: "|")
        let data = fingerprint.data(using: .utf8)!
        let hash = SHA256(data)
        
        return hash.toHexString()
    }
}
