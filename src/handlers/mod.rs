pub mod auth;
pub mod errors;
pub mod merchant;
pub mod payment;

use actix_web::{get, HttpResponse, Responder};
use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub redis: ConnectionManager,
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}
