use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::garage::models::{
    GarageLoginRequest,
    GarageLoginResponse,
    JobCreateRequest,
    JobStatusUpdateRequest,
    JobPartsAddRequest,
    JobPartUpdateRequest,
};
use crate::garage::repository::GarageRepo;

use crate::auth::create_token;

pub async fn login(
    state: web::Data<crate::state::AppState>,
    payload: web::Json<GarageLoginRequest>,
) -> actix_web::Result<HttpResponse> {
    let pool = &state.db;
    let req = payload.into_inner();

    // Look up garage user by username
    let user = match GarageRepo::find_user_by_username(pool, &req.username).await {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::Unauthorized().body("invalid credentials")),
    };

    // Basic checks: username present, active, and password match (DEV: raw compare)
    let stored_pass = user.password_hash.clone().unwrap_or_default();
    if !user.is_active || stored_pass != req.password {
        return Ok(HttpResponse::Unauthorized().body("invalid credentials"));
    }

    // Build JWT via centralized helper
    let token = create_token(
        user.id.to_string(),
        req.username.clone(),
        user.role.clone(),
        24,
    )
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("token creation error: {}", e)))?;

    let resp = GarageLoginResponse {
        token,
        id: user.id,
        username: req.username,
        display_name: user.display_name,
        role: user.role,
    };

    Ok(HttpResponse::Ok().json(resp))
}

// GET /api/garage/users/{user_id}/jobs
pub async fn list_jobs_for_user(
    state: web::Data<crate::state::AppState>,
    path: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let user_id_str = path.into_inner();
    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid user id")),
    };

    let rows = GarageRepo::list_jobs_for_garage_user(&state.db, user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Ok().json(rows))
}

// POST /api/garage/users/{user_id}/jobs
pub async fn create_job_for_user(
    state: web::Data<crate::state::AppState>,
    path: web::Path<String>,
    payload: web::Json<JobCreateRequest>,
) -> actix_web::Result<HttpResponse> {
    let user_id_str = path.into_inner();
    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid user id")),
    };

    let req = payload.into_inner();

    let created = GarageRepo::create_job_with_entities(&state.db, user_id, &req)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Created().json(created))
}

// GET /api/garage/jobs/{job_id}
pub async fn get_job_details(
    state: web::Data<crate::state::AppState>,
    path: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let job_id_str = path.into_inner();
    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid job id")),
    };

    let details = GarageRepo::get_job_details(&state.db, job_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Ok().json(details))
}

// PATCH /api/garage/jobs/{job_id}/status
pub async fn update_job_status(
    state: web::Data<crate::state::AppState>,
    path: web::Path<String>,
    payload: web::Json<JobStatusUpdateRequest>,
) -> actix_web::Result<HttpResponse> {
    let job_id_str = path.into_inner();
    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid job id")),
    };

    let body = payload.into_inner();

    let updated = GarageRepo::update_job_status(&state.db, job_id, &body)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Ok().json(updated))
}

// DELETE /api/garage/jobs/{job_id}/parts/{part_id}
pub async fn delete_job_part(
    state: web::Data<crate::state::AppState>,
    path: web::Path<(String, String)>,
) -> actix_web::Result<HttpResponse> {
    let (job_id_str, part_id_str) = path.into_inner();

    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid job id")),
    };
    let part_id = match Uuid::parse_str(&part_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid part id")),
    };

    let parts = GarageRepo::remove_job_part(&state.db, job_id, part_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Ok().json(parts))
}

// POST /api/garage/jobs/{job_id}/parts
pub async fn add_job_parts(
    state: web::Data<crate::state::AppState>,
    path: web::Path<String>,
    payload: web::Json<JobPartsAddRequest>,
) -> actix_web::Result<HttpResponse> {
    let job_id_str = path.into_inner();
    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid job id")),
    };

    let body = payload.into_inner();
    let parts = GarageRepo::add_job_parts(&state.db, job_id, &body.parts)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Ok().json(parts))
}

// PATCH /api/garage/jobs/{job_id}/parts/{part_id}
pub async fn update_job_part(
    state: web::Data<crate::state::AppState>,
    path: web::Path<(String, String)>,
    payload: web::Json<JobPartUpdateRequest>,
) -> actix_web::Result<HttpResponse> {
    let (job_id_str, part_id_str) = path.into_inner();
    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid job id")),
    };
    let part_id = match Uuid::parse_str(&part_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid part id")),
    };

    let req = payload.into_inner();
    let updated = GarageRepo::update_job_part(&state.db, job_id, part_id, &req)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("db error: {}", e)))?;

    Ok(HttpResponse::Ok().json(updated))
}
