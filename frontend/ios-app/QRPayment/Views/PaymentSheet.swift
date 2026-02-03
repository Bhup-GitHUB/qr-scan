import SwiftUI

struct PaymentSheet: View {
    let qrData: String
    
    @State private var amount = ""
    @State private var pin = ""
    @State private var isProcessing = false
    @State private var paymentResponse: PaymentInitResponse?
    @State private var errorMessage: String?
    @State private var showSuccess = false
    
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationStack {
            VStack(spacing: 18) {
                if let response = paymentResponse {
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Paying to")
                            .font(.headline)
                        
                        Text(response.merchant.name)
                            .font(.title2)
                            .fontWeight(.semibold)
                        
                        Text("Amount: ₹\(response.amount, specifier: "%.2f")")
                            .font(.title3)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
                    .background(.thinMaterial)
                    .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                    .padding(.horizontal)
                    
                    SecureField("Enter PIN", text: $pin)
                        .textFieldStyle(.roundedBorder)
                        .keyboardType(.numberPad)
                        .padding(.horizontal)
                    
                    Button("Confirm Payment") {
                        executePayment(sessionId: response.sessionId)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(isProcessing)
                    
                } else {
                    TextField("Enter Amount", text: $amount)
                        .textFieldStyle(.roundedBorder)
                        .keyboardType(.decimalPad)
                        .padding(.horizontal)
                    
                    Button("Continue") {
                        initiatePayment()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(isProcessing)
                }
                
                if let errorMessage {
                    Text(errorMessage)
                        .foregroundStyle(.red)
                        .font(.callout)
                        .padding(.horizontal)
                }
                
                if isProcessing {
                    ProgressView()
                }
                
                Spacer()
            }
            .padding(.top, 16)
            .navigationTitle("Payment")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Cancel") { dismiss() }
                }
            }
            .alert("Payment Successful", isPresented: $showSuccess) {
                Button("OK") { dismiss() }
            }
        }
    }
    
    private func initiatePayment() {
        guard let amountValue = Double(amount), amountValue > 0 else {
            errorMessage = "Invalid amount"
            return
        }
        
        isProcessing = true
        errorMessage = nil
        
        Task {
            do {
                let response = try await APIService.shared.initiatePayment(qrData: qrData, amount: amountValue)
                await MainActor.run {
                    paymentResponse = response
                    isProcessing = false
                }
            } catch {
                await MainActor.run {
                    errorMessage = "Failed to initiate payment"
                    isProcessing = false
                }
            }
        }
    }
    
    private func executePayment(sessionId: String) {
        let trimmedPin = pin.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedPin.isEmpty else {
            errorMessage = "Please enter PIN"
            return
        }
        
        isProcessing = true
        errorMessage = nil
        
        Task {
            do {
                _ = try await APIService.shared.executePayment(sessionId: sessionId, pin: trimmedPin)
                await MainActor.run {
                    isProcessing = false
                    showSuccess = true
                }
            } catch {
                await MainActor.run {
                    errorMessage = "Payment failed"
                    isProcessing = false
                }
            }
        }
    }
}

