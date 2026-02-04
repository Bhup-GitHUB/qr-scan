use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use uuid::Uuid;

use crate::config::Config;
use crate::handlers::errors::AppError;
use crate::services::auth;

#[derive(Clone)]
pub struct JwtAuth {
    pub config: Config,
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    config: Config,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let cfg = self.config.clone();
        let auth_header = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let srv = self.service.clone();

        Box::pin(async move {
            let auth_header = auth_header.ok_or_else(|| AppError::unauthorized("missing authorization"))?;
            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| AppError::unauthorized("invalid authorization"))?;
            let claims = auth::validate_token(&cfg, token)?;
            let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::unauthorized("invalid token"))?;

            req.extensions_mut().insert(AuthenticatedUser { user_id });
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}
