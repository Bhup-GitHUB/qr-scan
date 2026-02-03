import AVFoundation
import AudioToolbox
import UIKit

final class QRScannerService: NSObject, ObservableObject {
    @Published var scannedCode: String?
    @Published var isScanning = false
    
    private var captureSession: AVCaptureSession?
    private var previewLayer: AVCaptureVideoPreviewLayer?
    
    func startScanning(in view: UIView) {
        guard !isScanning else { return }
        
        let session = AVCaptureSession()
        captureSession = session
        
        guard let videoCaptureDevice = AVCaptureDevice.default(for: .video) else {
            return
        }
        
        let videoInput: AVCaptureDeviceInput
        
        do {
            videoInput = try AVCaptureDeviceInput(device: videoCaptureDevice)
        } catch {
            return
        }
        
        guard session.canAddInput(videoInput) else {
            return
        }
        session.addInput(videoInput)
        
        let metadataOutput = AVCaptureMetadataOutput()
        guard session.canAddOutput(metadataOutput) else {
            return
        }
        
        session.addOutput(metadataOutput)
        metadataOutput.setMetadataObjectsDelegate(self, queue: DispatchQueue.main)
        metadataOutput.metadataObjectTypes = [.qr]
        
        let layer = AVCaptureVideoPreviewLayer(session: session)
        layer.frame = view.layer.bounds
        layer.videoGravity = .resizeAspectFill
        previewLayer = layer
        view.layer.addSublayer(layer)
        
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            session.startRunning()
            DispatchQueue.main.async {
                self?.isScanning = true
            }
        }
    }
    
    func stopScanning() {
        captureSession?.stopRunning()
        previewLayer?.removeFromSuperlayer()
        isScanning = false
    }
}

extension QRScannerService: AVCaptureMetadataOutputObjectsDelegate {
    func metadataOutput(
        _ output: AVCaptureMetadataOutput,
        didOutput metadataObjects: [AVMetadataObject],
        from connection: AVCaptureConnection
    ) {
        guard let metadataObject = metadataObjects.first else { return }
        guard let readableObject = metadataObject as? AVMetadataMachineReadableCodeObject else { return }
        guard let stringValue = readableObject.stringValue else { return }
        
        AudioServicesPlaySystemSound(SystemSoundID(kSystemSoundID_Vibrate))
        scannedCode = stringValue
        stopScanning()
    }
}

