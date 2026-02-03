use actix_web::{post, web, HttpResponse};

use crate::handlers::errors::AppError;
use crate::handlers::AppState;
use crate::models::user::{LoginRequest, RegisterRequest};
use crate::services;

#[post("/register")]
pub async fn register(
    state: web::Data<AppState>,
    payload: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    let resp = services::auth::register(&state.config, &state.db, payload.into_inner()).await?;
    Ok(HttpResponse::Ok().json(resp))
}

#[post("/login")]
pub async fn login(
    state: web::Data<AppState>,
    payload: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let resp = services::auth::login(&state.config, &state.db, payload.into_inner()).await?;
    Ok(HttpResponse::Ok().json(resp))
}
