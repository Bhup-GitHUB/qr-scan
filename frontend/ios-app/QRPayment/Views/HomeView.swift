import SwiftUI

struct HomeView: View {
    @EnvironmentObject private var appState: AppState
    
    var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
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

