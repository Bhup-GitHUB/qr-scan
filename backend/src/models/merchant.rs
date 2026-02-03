use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Merchant {
    pub id: Uuid,
    pub name: String,
    pub upi_id: String,
    pub category: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub qr_code_data: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct QRScanRequest {
    pub qr_data: String,
}
