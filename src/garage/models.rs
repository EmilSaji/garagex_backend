use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct GarageUser {
    pub id: Uuid,
    pub garage_id: Uuid,
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct GarageLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct GarageLoginResponse {
    pub token: String,
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
}

// Slim job listing item for garage dashboard
#[derive(Debug, FromRow, Serialize)]
pub struct JobListItem {
    pub job_id: Uuid,
    pub vehicle_number: Option<String>,
    pub owner_name: Option<String>,
    pub estimated_delivery_date: Option<chrono::NaiveDate>,
    pub estimated_time: Option<String>,
    pub status: Option<String>,
}

// Request body to create a job
#[derive(Debug, Deserialize)]
pub struct JobCreateRequest {
    pub customer_name: Option<String>,
    pub phone: String,
    pub vehicle_number: String,
    pub vehicle_make: Option<String>,
    pub vehicle_model: Option<String>,
    pub complaint: Option<String>,
    pub estimated_delivery_date: Option<chrono::NaiveDate>,
    pub estimated_time: Option<String>,
}

// Response after creating a job
#[derive(Debug, Serialize)]
pub struct JobCreatedResponse {
    pub job_id: Uuid,
    pub job_identifier: String,
    pub vehicle_id: Uuid,
    pub customer_id: Uuid,
    pub vehicle_number: String,
    pub owner_name: Option<String>,
    pub estimated_delivery_date: Option<chrono::NaiveDate>,
    pub estimated_time: Option<String>,
    pub status: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct JobPartItem {
    pub id: Uuid,
    pub name: String,
    pub quantity: Option<i32>,
    pub unit_price: f64,
    pub tax_percent: Option<f64>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct JobStatusHistoryItem {
    pub id: Uuid,
    pub from_status: Option<String>,
    pub to_status: String,
    pub note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct JobDetailsResponse {
    pub job_id: Uuid,
    pub status: String,
    pub remarks: Option<String>,
    pub vehicle_number: Option<String>,
    pub vehicle_make: Option<String>,
    pub vehicle_model: Option<String>,
    pub owner_name: Option<String>,
    pub parts: Vec<JobPartItem>,
    pub status_history: Vec<JobStatusHistoryItem>,
}

// Part payload to create when updating job
#[derive(Debug, Deserialize)]
pub struct JobPartCreateItem {
    pub name: String,
    pub quantity: Option<i32>,
    pub unit_price: f64,
    pub tax_percent: Option<f64>,
}

// Request to update job status
#[derive(Debug, Deserialize)]
pub struct JobStatusUpdateRequest {
    pub to_status: String,
    pub note: Option<String>,
    pub remarks: Option<String>,
}

// Response after updating job status - return full status history
#[derive(Debug, Serialize)]
pub struct JobStatusUpdateResponse {
    pub status_history: Vec<JobStatusHistoryItem>,
}

// Request to add multiple parts to a job
#[derive(Debug, Deserialize)]
pub struct JobPartsAddRequest {
    pub parts: Vec<JobPartCreateItem>,
}

// Request to update a single part
#[derive(Debug, Deserialize)]
pub struct JobPartUpdateRequest {
    pub name: Option<String>,
    pub quantity: Option<i32>,
    pub unit_price: Option<f64>,
    pub tax_percent: Option<f64>,
}
