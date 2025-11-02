use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::env;

use crate::auth::extractor::Claims;

/// Create and sign a JWT token with the common Claims shape.
/// ttl_hours: how many hours the token should be valid from now.
pub fn create_token(
    sub: String,
    username: String,
    role: String,
    ttl_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into());
    let expiration = Utc::now() + Duration::hours(ttl_hours);

    let claims = Claims {
        sub,
        username,
        role,
        exp: expiration.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}
