use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::cache::redis_client::RedisClient;
use crate::handlers::errors::AppError;
use crate::models::payment::{
    MerchantInfo, PaymentExecuteRequest, PaymentExecuteResponse, PaymentInitRequest, PaymentInitResponse, Transaction,
    TransactionStatus,
};
use crate::models::user::User;
use crate::services::merchant;

pub async fn initiate_payment(
    db: &PgPool,
    redis: &RedisClient,
    user_id: Uuid,
    req: PaymentInitRequest,
) -> Result<PaymentInitResponse, AppError> {
    if req.amount <= 0.0 {
        return Err(AppError::bad_request("amount must be greater than 0"));
    }

    let cache_key = format!("payment:idempotency:{}", req.idempotency_key);
    if let Some(existing) = redis
        .get::<PaymentInitResponse>(&cache_key)
        .await
        .map_err(|e| AppError::internal(e))?
    {
        return Ok(existing);
    }

    let merchant = merchant::get_merchant_by_qr(db, redis, &req.qr_data).await?;

    let created: Result<Transaction, sqlx::Error> = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (user_id, merchant_id, amount, status, idempotency_key)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, merchant_id, amount, status, idempotency_key, upi_txn_id, error_message, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(merchant.id)
    .bind(req.amount)
    .bind(TransactionStatus::Initiated)
    .bind(&req.idempotency_key)
    .fetch_one(db)
    .await;

    let transaction = match created {
        Ok(t) => t,
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    let existing: Transaction = sqlx::query_as::<_, Transaction>(
                        r#"
                        SELECT id, user_id, merchant_id, amount, status, idempotency_key, upi_txn_id, error_message, created_at, updated_at
                        FROM transactions
                        WHERE idempotency_key = $1 AND user_id = $2
                        "#,
                    )
                    .bind(&req.idempotency_key)
                    .bind(user_id)
                    .fetch_one(db)
                    .await
                    .map_err(AppError::from_sqlx)?;
                    existing
                } else {
                    return Err(AppError::from_sqlx(e));
                }
            } else {
                return Err(AppError::from_sqlx(e));
            }
        }
    };

    let response = PaymentInitResponse {
        session_id: transaction.id,
        merchant: MerchantInfo {
            name: merchant.name,
            upi_id: merchant.upi_id,
            category: merchant.category,
        },
        amount: transaction.amount,
        status: "initiated".to_string(),
    };

    redis
        .set(&cache_key, &response, 600)
        .await
        .map_err(|e| AppError::internal(e))?;

    Ok(response)
}

pub async fn execute_payment(
    db: &PgPool,
    redis: &RedisClient,
    user_id: Uuid,
    req: PaymentExecuteRequest,
) -> Result<PaymentExecuteResponse, AppError> {
    let mut tx = db.begin().await.map_err(AppError::from_sqlx)?;

    let transaction: Transaction = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, user_id, merchant_id, amount, status, idempotency_key, upi_txn_id, error_message, created_at, updated_at
        FROM transactions
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(req.session_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from_sqlx)?;

    if !matches!(transaction.status, TransactionStatus::Initiated | TransactionStatus::Pending) {
        tx.commit().await.map_err(AppError::from_sqlx)?;
        return Ok(PaymentExecuteResponse {
            transaction_id: transaction.id,
            status: format!("{:?}", transaction.status).to_lowercase(),
            upi_txn_id: transaction.upi_txn_id,
            message: "transaction already processed".to_string(),
        });
    }

    verify_pin(&mut tx, user_id, &req.pin).await?;
    ensure_balance(&mut tx, user_id, transaction.amount).await?;

    let upi_txn_id = format!("UPI{}", Uuid::new_v4());

    sqlx::query(
        r#"
        UPDATE transactions
        SET status = $1, upi_txn_id = $2, error_message = NULL, updated_at = CURRENT_TIMESTAMP
        WHERE id = $3
        "#,
    )
    .bind(TransactionStatus::Success)
    .bind(&upi_txn_id)
    .bind(transaction.id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::from_sqlx)?;

    sqlx::query(
        r#"
        UPDATE users
        SET balance = balance - $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        "#,
    )
    .bind(transaction.amount)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::from_sqlx)?;

    tx.commit().await.map_err(AppError::from_sqlx)?;

    let idempotency_cache_key = format!("payment:idempotency:{}", transaction.idempotency_key);
    let _ = redis.delete(&idempotency_cache_key).await;

    Ok(PaymentExecuteResponse {
        transaction_id: transaction.id,
        status: "success".to_string(),
        upi_txn_id: Some(upi_txn_id),
        message: "payment successful".to_string(),
    })
}

async fn verify_pin(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    user_id: Uuid,
    pin: &str,
) -> Result<(), AppError> {
    let user: User = sqlx::query_as::<_, User>(
        r#"
        SELECT id, phone_number, upi_id, name, balance, pin_hash, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from_sqlx)?;

    let ok = bcrypt::verify(pin, &user.pin_hash).map_err(|_| AppError::internal("pin verify failed"))?;
    if !ok {
        return Err(AppError::unauthorized("invalid pin"));
    }
    Ok(())
}

async fn ensure_balance(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    user_id: Uuid,
    amount: f64,
) -> Result<(), AppError> {
    let balance: (f64,) = sqlx::query_as(
        r#"
        SELECT balance
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from_sqlx)?;

    if balance.0 < amount {
        return Err(AppError::bad_request("insufficient balance"));
    }
    Ok(())
}
