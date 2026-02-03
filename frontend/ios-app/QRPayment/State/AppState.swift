import Foundation

@MainActor
final class AppState: ObservableObject {
    @Published var isAuthenticated: Bool
    
    init() {
        isAuthenticated = UserDefaults.standard.string(forKey: "auth_token")?.isEmpty == false
    }
    
    func setAuthToken(_ token: String) {
        UserDefaults.standard.set(token, forKey: "auth_token")
        isAuthenticated = true
    }
    
    func clearAuthToken() {
        UserDefaults.standard.removeObject(forKey: "auth_token")
        isAuthenticated = false
    }
}

