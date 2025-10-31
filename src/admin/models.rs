use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use serde_json::Value as JsonValue;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct NewGarage {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct GarageUser {
    pub id: Uuid,
    pub garage_id: Uuid,
    // username / password_hash allowed to be NULL for placeholder accounts
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub metadata: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
