use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::redis_client::RedisClient;
use crate::handlers::errors::AppError;
use crate::models::merchant::Merchant;

pub async fn get_merchant_by_qr(db: &PgPool, redis: &RedisClient, qr_data: &str) -> Result<Merchant, AppError> {
    let cache_key = format!("merchant:qr:{}", qr_data);

    if let Some(merchant) = redis
        .get::<Merchant>(&cache_key)
        .await
        .map_err(|e| AppError::internal(e))?
    {
        return Ok(merchant);
    }

    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, name, upi_id, category, address, phone, qr_code_data, created_at
        FROM merchants
        WHERE qr_code_data = $1
        "#,
    )
    .bind(qr_data)
    .fetch_one(db)
    .await
    .map_err(AppError::from_sqlx)?;

    redis
        .set(&cache_key, &merchant, 3600)
        .await
        .map_err(|e| AppError::internal(e))?;

    Ok(merchant)
}

pub async fn get_merchant_by_id(db: &PgPool, redis: &RedisClient, merchant_id: Uuid) -> Result<Merchant, AppError> {
    let cache_key = format!("merchant:id:{}", merchant_id);

    if let Some(merchant) = redis
        .get::<Merchant>(&cache_key)
        .await
        .map_err(|e| AppError::internal(e))?
    {
        return Ok(merchant);
    }

    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, name, upi_id, category, address, phone, qr_code_data, created_at
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_one(db)
    .await
    .map_err(AppError::from_sqlx)?;

    redis
        .set(&cache_key, &merchant, 3600)
        .await
        .map_err(|e| AppError::internal(e))?;

    Ok(merchant)
}

