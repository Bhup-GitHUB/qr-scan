use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::handlers::errors::AppError;
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest, User, UserPublic};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

pub async fn register(cfg: &Config, db: &PgPool, req: RegisterRequest) -> Result<AuthResponse, AppError> {
    if req.pin.len() < 4 || req.pin.len() > 12 {
        return Err(AppError::bad_request("pin must be 4-12 digits"));
    }

    let pin_hash = hash(req.pin, DEFAULT_COST).map_err(|_| AppError::internal("hash failed"))?;

    let user: User = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (phone_number, upi_id, name, pin_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, phone_number, upi_id, name, balance, pin_hash, created_at, updated_at
        "#,
    )
    .bind(req.phone_number)
    .bind(req.upi_id)
    .bind(req.name)
    .bind(pin_hash)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::from_sqlx(e))?;

    let (token, expires_in) = mint_token(cfg, user.id)?;
    Ok(AuthResponse {
        token,
        user: UserPublic::from(user),
        expires_in,
    })
}

pub async fn login(cfg: &Config, db: &PgPool, req: LoginRequest) -> Result<AuthResponse, AppError> {
    let user: User = sqlx::query_as::<_, User>(
        r#"
        SELECT id, phone_number, upi_id, name, balance, pin_hash, created_at, updated_at
        FROM users
        WHERE phone_number = $1
        "#,
    )
    .bind(req.phone_number)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::from_sqlx(e))?;

    let ok = verify(req.pin, &user.pin_hash).map_err(|_| AppError::internal("verify failed"))?;
    if !ok {
        return Err(AppError::unauthorized("invalid credentials"));
    }

    let (token, expires_in) = mint_token(cfg, user.id)?;
    Ok(AuthResponse {
        token,
        user: UserPublic::from(user),
        expires_in,
    })
}

fn mint_token(cfg: &Config, user_id: Uuid) -> Result<(String, i64), AppError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(cfg.jwt_ttl_seconds);
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::internal("jwt encode failed"))?;

    Ok((token, cfg.jwt_ttl_seconds))
}

pub fn validate_token(cfg: &Config, token: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::unauthorized("invalid token"))?;
    Ok(data.claims)
}
