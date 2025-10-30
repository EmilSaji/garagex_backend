use sqlx::PgPool;

/// The application state shared across handlers.
/// Wrap in Arc in lib.rs to clone cheaply into actix Data.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    // add other shared clients like whatsapp_client, redis_client, etc.
}
