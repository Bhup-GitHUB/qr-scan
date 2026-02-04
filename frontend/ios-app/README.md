# iOS App (SwiftUI)

This folder contains the SwiftUI source files for the QRPayment iOS app.

## Create the Xcode project

1. Open Xcode
2. File > New > Project > iOS > App
3. Name: QRPayment
4. Interface: SwiftUI
5. Language: Swift

## Add files

Copy the `QRPayment/` folder contents into your Xcode project (or add the folder to the project).

## Configuration

- Set `API_BASE_URL` in the app Info.plist to your backend URL.
- Ensure `NSCameraUsageDescription` exists in the app Info.plist.

## Backend (Docker)

From repo root:

- Start stack: `docker-compose up -d --build`
- Seed merchant: `docker-compose exec -T postgres psql -U qrpay -d qr_payment -f /seed/seed.sql`

Register a user via API:

- `POST http://localhost:8080/auth/register` with JSON `{ "phone_number": "9876543210", "upi_id": "testuser@paytm", "name": "Test User", "pin": "1234" }`
