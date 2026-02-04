use actix_web::{post, web, HttpRequest, HttpResponse};

use crate::handlers::errors::AppError;
use crate::handlers::AppState;
use crate::middleware::jwt_auth::AuthenticatedUser;
use crate::models::payment::{PaymentExecuteRequest, PaymentInitRequest};
use crate::services;

#[post("/payment/initiate")]
pub async fn initiate_payment(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<PaymentInitRequest>,
) -> Result<HttpResponse, AppError> {
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| AppError::unauthorized("unauthorized"))?
        .user_id;

    let resp = services::payment::initiate_payment(&state.db, &state.redis, user, payload.into_inner()).await?;
    Ok(HttpResponse::Ok().json(resp))
}

#[post("/payment/execute")]
pub async fn execute_payment(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<PaymentExecuteRequest>,
) -> Result<HttpResponse, AppError> {
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| AppError::unauthorized("unauthorized"))?
        .user_id;

    let resp = services::payment::execute_payment(&state.db, &state.redis, user, payload.into_inner()).await?;
    Ok(HttpResponse::Ok().json(resp))
}

