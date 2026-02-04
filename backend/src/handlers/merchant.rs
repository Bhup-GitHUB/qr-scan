use actix_web::{post, web, HttpResponse};

use crate::handlers::errors::AppError;
use crate::handlers::AppState;
use crate::models::merchant::QRScanRequest;
use crate::services;

#[post("/merchant/resolve")]
pub async fn resolve_merchant(
    state: web::Data<AppState>,
    payload: web::Json<QRScanRequest>,
) -> Result<HttpResponse, AppError> {
    let merchant = services::merchant::get_merchant_by_qr(&state.db, &state.redis, &payload.qr_data).await?;
    Ok(HttpResponse::Ok().json(merchant))
}

