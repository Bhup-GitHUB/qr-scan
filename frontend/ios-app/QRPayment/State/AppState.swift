import Foundation

@MainActor
final class AppState: ObservableObject {
    @Published var isAuthenticated: Bool
    @Published var currentUser: UserPublic?
    
    init() {
        isAuthenticated = UserDefaults.standard.string(forKey: "auth_token")?.isEmpty == false
        currentUser = nil
    }
    
    func setAuthenticated(user: UserPublic, token: String) {
        UserDefaults.standard.set(token, forKey: "auth_token")
        isAuthenticated = true
        currentUser = user
    }
    
    func clearAuthToken() {
        UserDefaults.standard.removeObject(forKey: "auth_token")
        isAuthenticated = false
        currentUser = nil
    }
}
