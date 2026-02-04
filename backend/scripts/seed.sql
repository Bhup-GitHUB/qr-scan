INSERT INTO merchants (name, upi_id, category, phone, qr_code_data)
VALUES (
  'Coffee Shop',
  'coffeeshop@upi',
  'food',
  '9999999999',
  'upi://pay?pa=coffeeshop@upi&pn=Coffee%20Shop&am=100'
)
ON CONFLICT DO NOTHING;
