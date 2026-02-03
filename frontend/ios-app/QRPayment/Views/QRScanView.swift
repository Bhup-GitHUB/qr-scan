import SwiftUI

struct QRScanView: View {
    @StateObject private var scanner = QRScannerService()
    @State private var showPaymentSheet = false
    @State private var qrData = ""
    
    var body: some View {
        ZStack {
            CameraPreview(scanner: scanner)
                .ignoresSafeArea()
            
            VStack(spacing: 0) {
                Text("Scan QR Code")
                    .font(.title2)
                    .fontWeight(.semibold)
                    .foregroundStyle(.white)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 10)
                    .background(.black.opacity(0.7))
                    .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                    .padding(.top, 18)
                
                Spacer()
                
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(.white, lineWidth: 3)
                    .frame(width: 250, height: 250)
                
                Spacer()
            }
            .padding(.horizontal)
        }
        .onChange(of: scanner.scannedCode) { newValue in
            guard let code = newValue, !code.isEmpty else { return }
            qrData = code
            showPaymentSheet = true
        }
        .onDisappear {
            scanner.stopScanning()
        }
        .sheet(isPresented: $showPaymentSheet) {
            PaymentSheet(qrData: qrData)
        }
    }
}

struct CameraPreview: UIViewRepresentable {
    @ObservedObject var scanner: QRScannerService
    
    func makeUIView(context: Context) -> UIView {
        let view = UIView(frame: .zero)
        scanner.startScanning(in: view)
        return view
    }
    
    func updateUIView(_ uiView: UIView, context: Context) {}
    
    static func dismantleUIView(_ uiView: UIView, coordinator: ()) {
        uiView.layer.sublayers?.forEach { $0.removeFromSuperlayer() }
    }
}
