use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: Option<String>,
    pub phone: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AdminLoginResponse {
    pub token: String,
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Garage {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
