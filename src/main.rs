mod cache;
mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use actix_web::{middleware::Logger, web, App, HttpServer};
use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let cfg = Config::from_env().expect("failed to load config");
    let db = db::pool::create_pool(&cfg.database_url)
        .await
        .expect("failed to create db pool");
    let redis = cache::redis_client::create_redis_client(&cfg.redis_url)
        .await
        .expect("failed to create redis client");

    let state = handlers::AppState {
        config: cfg.clone(),
        db,
        redis,
    };

    let bind_addr = format!("{}:{}", cfg.server_host, cfg.server_port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Logger::default())
            .service(handlers::health)
            .service(
                web::scope("/auth")
                    .service(handlers::auth::register)
                    .service(handlers::auth::login),
            )
    })
    .bind(bind_addr)?
    .run()
    .await
}
