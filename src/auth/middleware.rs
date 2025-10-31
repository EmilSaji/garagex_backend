use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, Algorithm, Validation};
use std::{env, rc::Rc};

use crate::auth::extractor::Claims;

/// Middleware that verifies a Bearer JWT and inserts Claims into request extensions.
/// Construct with `AuthMiddleware::default()` (reads JWT_SECRET from env).
#[derive(Clone)]
pub struct AuthMiddleware {
    // keep secret bytes in Rc so cloning is cheap
    secret: Rc<Vec<u8>>,
    validation: Validation,
}

impl AuthMiddleware {
    pub fn new(secret_bytes: Vec<u8>) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        Self {
            secret: Rc::new(secret_bytes),
            validation,
        }
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string());
        Self::new(secret.into_bytes())
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: std::rc::Rc::new(service),
            secret: self.secret.clone(),
            validation: self.validation.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: std::rc::Rc<S>,
    secret: Rc<Vec<u8>>,
    validation: Validation,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let secret = self.secret.clone();
        let validation = self.validation.clone();

        Box::pin(async move {
            // Read Authorization header
            let auth_header = req
                .headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let token_opt = match auth_header {
                Some(h) if h.starts_with("Bearer ") => {
                    Some(h.trim_start_matches("Bearer ").trim().to_string())
                }
                _ => None,
            };

            let token = match token_opt {
                Some(t) if !t.is_empty() => t,
                _ => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "missing or invalid authorization header",
                    ));
                }
            };

            // Build decoding key from secret bytes
            let decoding_key = jsonwebtoken::DecodingKey::from_secret(&secret);

            // decode token
            let decoded = decode::<Claims>(&token, &decoding_key, &validation)
                .map_err(|_e| actix_web::error::ErrorUnauthorized("invalid token"))?;

            // insert claims into request extensions so extractors can pick it up
            req.extensions_mut().insert::<Claims>(decoded.claims);

            // call inner service
            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}
