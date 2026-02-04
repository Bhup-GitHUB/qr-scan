import SwiftUI

struct HomeView: View {
    @EnvironmentObject private var appState: AppState
    
    var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                if let user = appState.currentUser {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(user.name)
                            .font(.title2)
                            .fontWeight(.semibold)
                        Text("Balance: ₹\(user.balance, specifier: "%.2f")")
                            .font(.headline)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
                    .background(.thinMaterial)
                    .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }
                
                NavigationLink("Scan QR Code") {
                    QRScanView()
                }
                .buttonStyle(.borderedProminent)
                
                Button("Logout") {
                    appState.clearAuthToken()
                }
                .buttonStyle(.bordered)
                
                Spacer()
            }
            .padding()
            .navigationTitle("QRPayment")
        }
    }
}
