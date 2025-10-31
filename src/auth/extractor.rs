use actix_web::{FromRequest, HttpRequest, dev::Payload, Error, HttpMessage};
use futures_util::future::{ready, Ready};
use serde::{Deserialize, Serialize};

/// JWT Claims shape. Must match what you sign during login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

/// Simple extractor that pulls `Claims` from request extensions (populated by middleware).
/// Usage in handlers: `claims: AuthClaims`
#[derive(Debug, Clone)]
pub struct AuthClaims(pub Claims);

impl FromRequest for AuthClaims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // HttpMessage trait (imported above) provides `extensions()`.
        if let Some(claims) = req.extensions().get::<Claims>() {
            return ready(Ok(AuthClaims(claims.clone())));
        }
        ready(Err(actix_web::error::ErrorUnauthorized("missing auth claims")))
    }
}
