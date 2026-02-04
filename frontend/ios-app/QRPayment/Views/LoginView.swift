import SwiftUI

struct LoginView: View {
    @EnvironmentObject private var appState: AppState
    
    @State private var phoneNumber = ""
    @State private var pin = ""
    @State private var isLoading = false
    @State private var errorMessage: String?
    
    var body: some View {
        NavigationStack {
            VStack(spacing: 20) {
                Text("QR Payment")
                    .font(.largeTitle)
                    .fontWeight(.bold)
                
                TextField("Phone Number", text: $phoneNumber)
                    .textFieldStyle(.roundedBorder)
                    .keyboardType(.phonePad)
                    .textContentType(.telephoneNumber)
                    .padding(.horizontal)
                
                SecureField("PIN", text: $pin)
                    .textFieldStyle(.roundedBorder)
                    .keyboardType(.numberPad)
                    .textContentType(.oneTimeCode)
                    .padding(.horizontal)
                
                if let errorMessage {
                    Text(errorMessage)
                        .foregroundStyle(.red)
                        .font(.caption)
                        .padding(.horizontal)
                }
                
                Button(action: login) {
                    if isLoading {
                        ProgressView()
                    } else {
                        Text("Login")
                            .fontWeight(.semibold)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding()
                .background(Color.blue)
                .foregroundStyle(.white)
                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                .padding(.horizontal)
                .disabled(isLoading)
                
                Spacer()
            }
            .padding(.top, 24)
        }
    }
    
    private func login() {
        let trimmedPhone = phoneNumber.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedPin = pin.trimmingCharacters(in: .whitespacesAndNewlines)
        
        guard !trimmedPhone.isEmpty, !trimmedPin.isEmpty else {
            errorMessage = "Please fill all fields"
            return
        }
        
        isLoading = true
        errorMessage = nil
        
        Task {
            do {
                let response = try await APIService.shared.login(phoneNumber: trimmedPhone, pin: trimmedPin)
                await MainActor.run {
                    appState.setAuthenticated(user: response.user, token: response.token)
                    isLoading = false
                }
            } catch {
                await MainActor.run {
                    errorMessage = "Login failed"
                    isLoading = false
                }
            }
        }
    }
}
