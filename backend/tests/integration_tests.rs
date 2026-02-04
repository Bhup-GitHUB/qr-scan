use actix_web::{test, web, App};
use qr_payment_backend::cache::redis_client::RedisClient;
use qr_payment_backend::config::Config;
use qr_payment_backend::handlers;
use qr_payment_backend::middleware::jwt_auth::JwtAuth;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup() -> (Config, PgPool, RedisClient) {
    let cfg = Config::from_env().expect("failed to load config");
    let db = qr_payment_backend::db::pool::create_pool(&cfg.database_url)
        .await
        .expect("failed to create db pool");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    sqlx::query("TRUNCATE TABLE transactions, merchants, users CASCADE")
        .execute(&db)
        .await
        .expect("failed to reset tables");

    let redis = RedisClient::new(&cfg.redis_url)
        .await
        .expect("failed to create redis client");

    (cfg, db, redis)
}

async fn init_app(cfg: Config, db: PgPool, redis: RedisClient) -> impl actix_web::dev::Service<
    actix_web::dev::ServiceRequest,
    Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
    Error = actix_web::Error,
> {
    let state = handlers::AppState {
        config: cfg.clone(),
        db,
        redis,
    };

    let jwt = JwtAuth { config: cfg };

    test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(handlers::health)
            .service(
                web::scope("/auth")
                    .service(handlers::auth::register)
                    .service(handlers::auth::login),
            )
            .service(
                web::scope("/api")
                    .wrap(jwt)
                    .service(handlers::merchant::resolve_merchant)
                    .service(handlers::payment::initiate_payment)
                    .service(handlers::payment::execute_payment),
            ),
    )
    .await
}

#[actix_web::test]
async fn auth_and_payment_flow() {
    let (cfg, db, redis) = setup().await;

    let merchant_qr = "upi://pay?pa=coffeeshop@upi&pn=Coffee%20Shop&am=100";
    sqlx::query(
        r#"
        INSERT INTO merchants (name, upi_id, category, phone, qr_code_data)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind("Coffee Shop")
    .bind("coffeeshop@upi")
    .bind("food")
    .bind("9999999999")
    .bind(merchant_qr)
    .execute(&db)
    .await
    .expect("failed to seed merchant");

    let mut app = init_app(cfg.clone(), db.clone(), redis).await;

    let register_req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "phone_number": "9876543210",
            "upi_id": "testuser@paytm",
            "name": "Test User",
            "pin": "1234"
        }))
        .to_request();

    let register_resp: serde_json::Value = test::call_and_read_body_json(&mut app, register_req).await;
    let token = register_resp
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    sqlx::query("UPDATE users SET balance = 1000.0 WHERE phone_number = $1")
        .bind("9876543210")
        .execute(&db)
        .await
        .expect("failed to set balance");

    let init_req = test::TestRequest::post()
        .uri("/api/payment/initiate")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "qr_data": merchant_qr,
            "amount": 100.0,
            "idempotency_key": "unique-key-123"
        }))
        .to_request();

    let init_resp: serde_json::Value = test::call_and_read_body_json(&mut app, init_req).await;
    let session_id = init_resp
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| Uuid::parse_str(s).unwrap())
        .unwrap();

    let exec_req = test::TestRequest::post()
        .uri("/api/payment/execute")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "session_id": session_id,
            "pin": "1234"
        }))
        .to_request();

    let exec_resp: serde_json::Value = test::call_and_read_body_json(&mut app, exec_req).await;
    assert_eq!(exec_resp.get("status").and_then(|v| v.as_str()), Some("success"));
}

#[actix_web::test]
async fn payment_idempotency() {
    let (cfg, db, redis) = setup().await;

    let merchant_qr = "upi://pay?pa=coffeeshop@upi&pn=Coffee%20Shop&am=100";
    sqlx::query(
        r#"
        INSERT INTO merchants (name, upi_id, category, phone, qr_code_data)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind("Coffee Shop")
    .bind("coffeeshop@upi")
    .bind("food")
    .bind("9999999999")
    .bind(merchant_qr)
    .execute(&db)
    .await
    .expect("failed to seed merchant");

    let mut app = init_app(cfg.clone(), db.clone(), redis).await;

    let register_req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "phone_number": "9876543210",
            "upi_id": "testuser@paytm",
            "name": "Test User",
            "pin": "1234"
        }))
        .to_request();

    let register_resp: serde_json::Value = test::call_and_read_body_json(&mut app, register_req).await;
    let token = register_resp
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    sqlx::query("UPDATE users SET balance = 1000.0 WHERE phone_number = $1")
        .bind("9876543210")
        .execute(&db)
        .await
        .expect("failed to set balance");

    let idempotency_key = "same-key";
    let init_req_1 = test::TestRequest::post()
        .uri("/api/payment/initiate")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "qr_data": merchant_qr,
            "amount": 100.0,
            "idempotency_key": idempotency_key
        }))
        .to_request();

    let init_resp_1: serde_json::Value = test::call_and_read_body_json(&mut app, init_req_1).await;
    let session_id_1 = init_resp_1.get("session_id").and_then(|v| v.as_str()).unwrap().to_string();

    let init_req_2 = test::TestRequest::post()
        .uri("/api/payment/initiate")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "qr_data": merchant_qr,
            "amount": 100.0,
            "idempotency_key": idempotency_key
        }))
        .to_request();

    let init_resp_2: serde_json::Value = test::call_and_read_body_json(&mut app, init_req_2).await;
    let session_id_2 = init_resp_2.get("session_id").and_then(|v| v.as_str()).unwrap().to_string();

    assert_eq!(session_id_1, session_id_2);
}
