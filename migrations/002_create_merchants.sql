CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS merchants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    upi_id VARCHAR(100) UNIQUE NOT NULL,
    category VARCHAR(50),
    address TEXT,
    phone VARCHAR(15),
    qr_code_data TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_merchants_upi ON merchants(upi_id);
CREATE INDEX IF NOT EXISTS idx_merchants_qr ON merchants(qr_code_data);
