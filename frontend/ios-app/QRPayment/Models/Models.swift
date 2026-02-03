import Foundation

struct AuthResponse: Codable {
    let token: String
    let user: UserPublic
    let expiresIn: Int
}

struct UserPublic: Codable {
    let id: String
    let name: String
    let upiId: String
    let balance: Double
}

struct PaymentInitResponse: Codable {
    let sessionId: String
    let merchant: MerchantInfo
    let amount: Double
    let status: String
}

struct MerchantInfo: Codable {
    let name: String
    let upiId: String
    let category: String?
}

struct PaymentExecuteResponse: Codable {
    let transactionId: String
    let status: String
    let upiTxnId: String?
    let message: String
}

