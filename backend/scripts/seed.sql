INSERT INTO users (phone_number, upi_id, name, balance, pin_hash)
VALUES (
  '9876543210',
  'testuser@paytm',
  'Test User',
  10000.00,
  '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5iXL0.eG5BZGy'
)
ON CONFLICT DO NOTHING;

INSERT INTO merchants (name, upi_id, category, phone, qr_code_data)
VALUES (
  'Coffee Shop',
  'coffeeshop@upi',
  'food',
  '9999999999',
  'upi://pay?pa=coffeeshop@upi&pn=Coffee%20Shop&am=100'
)
ON CONFLICT DO NOTHING;

