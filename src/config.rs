use serde::Deserialize;
use std::env;

/// App configuration loaded from environment (.env via dotenvy)
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub env: String,
}

impl Config {
    pub fn from_env() -> Self {
        // attempt to load .env file in working directory
        dotenvy::dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env or environment");

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(3001);
        let env = env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        Self {
            database_url,
            host,
            port,
            env,
        }
    }
}
