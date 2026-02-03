import Foundation

enum APIError: Error {
    case invalidURL
    case invalidResponse
    case requestFailed(Error)
    case encodingFailed(Error)
    case decodingFailed(Error)
}

final class APIService {
    static let shared = APIService()
    
    private let baseURL: String
    private var authToken: String?
    
    private init() {
        baseURL = (Bundle.main.object(forInfoDictionaryKey: "API_BASE_URL") as? String)?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false
            ? (Bundle.main.object(forInfoDictionaryKey: "API_BASE_URL") as? String ?? "http://localhost:8080")
            : "http://localhost:8080"
        authToken = UserDefaults.standard.string(forKey: "auth_token")
    }
    
    func login(phoneNumber: String, pin: String) async throws -> AuthResponse {
        let endpoint = "\(baseURL)/auth/login"
        let body: [String: Any] = [
            "phone_number": phoneNumber,
            "pin": pin
        ]
        
        let response: AuthResponse = try await post(endpoint: endpoint, body: body)
        authToken = response.token
        UserDefaults.standard.set(response.token, forKey: "auth_token")
        return response
    }
    
    func initiatePayment(qrData: String, amount: Double) async throws -> PaymentInitResponse {
        let endpoint = "\(baseURL)/api/payment/initiate"
        let body: [String: Any] = [
            "qr_data": qrData,
            "amount": amount,
            "idempotency_key": UUID().uuidString
        ]
        
        return try await post(endpoint: endpoint, body: body, requiresAuth: true)
    }
    
    func executePayment(sessionId: String, pin: String) async throws -> PaymentExecuteResponse {
        let endpoint = "\(baseURL)/api/payment/execute"
        let body: [String: Any] = [
            "session_id": sessionId,
            "pin": pin
        ]
        
        return try await post(endpoint: endpoint, body: body, requiresAuth: true)
    }
    
    private func post<T: Decodable>(
        endpoint: String,
        body: [String: Any],
        requiresAuth: Bool = false
    ) async throws -> T {
        guard let url = URL(string: endpoint) else {
            throw APIError.invalidURL
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        if requiresAuth {
            if let token = authToken ?? UserDefaults.standard.string(forKey: "auth_token") {
                request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
            }
        }
        
        do {
            request.httpBody = try JSONSerialization.data(withJSONObject: body, options: [])
        } catch {
            throw APIError.encodingFailed(error)
        }
        
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            
            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIError.invalidResponse
            }
            
            guard (200...299).contains(httpResponse.statusCode) else {
                throw APIError.invalidResponse
            }
            
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            
            do {
                return try decoder.decode(T.self, from: data)
            } catch {
                throw APIError.decodingFailed(error)
            }
        } catch {
            throw APIError.requestFailed(error)
        }
    }
}

