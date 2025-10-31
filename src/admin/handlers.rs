use actix_web::{web, Error, HttpResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

use crate::admin::models::{AdminLoginRequest, AdminLoginResponse, Garage, NewGarage};
use crate::admin::repository::{AdminRepo, GarageRepo};

// Auth extractor
use crate::auth::AuthClaims;

#[derive(Serialize, Deserialize, Clone)]
struct ClaimsLocal {
    sub: String,
    username: String,
    role: String,
    exp: usize,
}

/// Public login handler
pub async fn login(
    state: web::Data<crate::state::AppState>,
    payload: web::Json<AdminLoginRequest>,
) -> actix_web::Result<HttpResponse> {
    let pool = &state.db;
    let req = payload.into_inner();

    // Look up admin by username
    let admin = match AdminRepo::find_by_username(pool, &req.username).await {
        Ok(a) => a,
        Err(_) => return Ok(HttpResponse::Unauthorized().body("invalid credentials")),
    };

    // DEV: raw password compare
    let stored = admin.password_hash.clone().unwrap_or_default();
    if stored != req.password {
        return Ok(HttpResponse::Unauthorized().body("invalid credentials"));
    }

    // Build JWT
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into());
    let expiration = Utc::now() + Duration::hours(24);
    let claims = ClaimsLocal {
        sub: admin.id.to_string(),
        username: admin.username.clone(),
        role: "ADMIN".to_string(),
        exp: expiration.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("token creation error: {}", e))
    })?;

    let resp = AdminLoginResponse {
        token,
        id: admin.id,
        username: admin.username,
        display_name: admin.display_name,
    };

    Ok(HttpResponse::Ok().json(resp))
}

/// GET /api/admin/garages?q=...&limit=20
pub async fn list_garages(
    _claims: AuthClaims,
    state: web::Data<crate::state::AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let q = query.get("q").map(|s| s.as_str());
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);

    let garages = GarageRepo::list_garages(&state.db, q, limit)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Ok().json(garages))
}

/// GET /api/admin/garages/{id}
pub async fn get_garage(
    _claims: AuthClaims,
    state: web::Data<crate::state::AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let id_str = path.into_inner();
    let id =
        Uuid::parse_str(&id_str).map_err(|_| actix_web::error::ErrorBadRequest("invalid id"))?;

    match GarageRepo::get_garage_by_id(&state.db, id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?
    {
        Some(g) => Ok(HttpResponse::Ok().json(g)),
        None => Err(actix_web::error::ErrorNotFound("garage not found")),
    }
}

pub async fn add_garage(
    _claims: AuthClaims,
    state: web::Data<crate::state::AppState>,
    payload: web::Json<NewGarage>,
) -> Result<HttpResponse, Error> {
    let pool = &state.db;
    let new = payload.into_inner();

    // create garage and placeholder garage user atomically
    let (created_garage, created_user) = GarageRepo::add_garage_with_admin(pool, &new)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    // Build response - return garage and the created user id (no password_hash)
    #[derive(serde::Serialize)]
    struct Resp {
        garage: Garage,
        admin_user_id: Uuid,
    }

    let resp = Resp {
        garage: created_garage,
        admin_user_id: created_user.id,
    };

    Ok(HttpResponse::Created().json(resp))
}

pub async fn delete_garage(
    _claims: AuthClaims,
    state: web::Data<crate::state::AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let id_str = path.into_inner();
    let id =
        Uuid::parse_str(&id_str).map_err(|_| actix_web::error::ErrorBadRequest("invalid id"))?;

    match GarageRepo::delete_garage_by_id(&state.db, id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?
    {
        Some(g) => Ok(HttpResponse::Ok().json(g)),
        None => Err(actix_web::error::ErrorNotFound(
            "garage not found or already deleted",
        )),
    }
}
