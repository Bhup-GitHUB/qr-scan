use actix_web::{middleware::Logger, web, App, HttpServer};
use qr_payment_backend::config::Config;
use qr_payment_backend::middleware::jwt_auth::JwtAuth;
use qr_payment_backend::{cache, db, handlers};
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let cfg = Config::from_env().expect("failed to load config");
    let db = connect_db_with_retry(&cfg.database_url).await;
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    let redis = connect_redis_with_retry(&cfg.redis_url).await;

    let state = handlers::AppState {
        config: cfg.clone(),
        db,
        redis,
    };

    let bind_addr = format!("{}:{}", cfg.server_host, cfg.server_port);

    HttpServer::new(move || {
        let jwt = JwtAuth {
            config: state.config.clone(),
        };

        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Logger::default())
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
            )
    })
    .bind(bind_addr)?
    .run()
    .await
}

async fn connect_db_with_retry(database_url: &str) -> sqlx::PgPool {
    let mut last_error: Option<sqlx::Error> = None;
    for _ in 0..60 {
        match db::pool::create_pool(database_url).await {
            Ok(pool) => return pool,
            Err(e) => {
                last_error = Some(e);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    panic!(
        "failed to connect to database: {}",
        last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
}

async fn connect_redis_with_retry(redis_url: &str) -> cache::redis_client::RedisClient {
    let mut last_error: Option<String> = None;
    for _ in 0..60 {
        match cache::redis_client::RedisClient::new(redis_url).await {
            Ok(client) => return client,
            Err(e) => {
                last_error = Some(e.to_string());
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    panic!(
        "failed to connect to redis: {}",
        last_error.unwrap_or_else(|| "unknown".to_string())
    );
}
