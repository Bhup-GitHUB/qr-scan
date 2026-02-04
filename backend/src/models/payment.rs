use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
pub enum TransactionStatus {
    Initiated,
    Pending,
    Success,
    Failed,
    Refunded,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub merchant_id: Uuid,
    pub amount: f64,
    pub status: TransactionStatus,
    pub idempotency_key: String,
    pub upi_txn_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct PaymentInitRequest {
    pub qr_data: String,
    pub amount: f64,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentInitResponse {
    pub session_id: Uuid,
    pub merchant: MerchantInfo,
    pub amount: f64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantInfo {
    pub name: String,
    pub upi_id: String,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaymentExecuteRequest {
    pub session_id: Uuid,
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentExecuteResponse {
    pub transaction_id: Uuid,
    pub status: String,
    pub upi_txn_id: Option<String>,
    pub message: String,
}
