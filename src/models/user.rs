use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub phone_number: String,
    pub upi_id: String,
    pub name: String,
    pub balance: f64,
    #[serde(skip_serializing)]
    pub pin_hash: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub name: String,
    pub upi_id: String,
    pub balance: f64,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        UserPublic {
            id: u.id,
            name: u.name,
            upi_id: u.upi_id,
            balance: u.balance,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub phone_number: String,
    pub upi_id: String,
    pub name: String,
    pub pin: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub phone_number: String,
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
    pub expires_in: i64,
}
