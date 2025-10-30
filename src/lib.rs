pub mod config;
pub mod health;
pub mod routes;
pub mod state;

use crate::config::Config;
use crate::state::AppState;
use actix_web::{App, HttpServer};
use eyre::Result;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn run(cfg: Config) -> Result<()> {
    // Connect to DB (tune PgPoolOptions if needed)
    let pool = PgPool::connect(&cfg.database_url)
        .await
        .map_err(|e| eyre::eyre!("DB connect error: {}", e))?;

    // Run migrations located at project root ./migrations
    // Ensure your ./migrations exists relative to where cargo run executes.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| eyre::eyre!("Migrations failed: {}", e))?;

    // Build state and share it with handlers
    let state = AppState {
        db: pool,
        // add other shared clients (whatsapp, redis, etc.) here
    };
    let shared_state = Arc::new(state);

    // Build server app factory closure
    let bind_addr = (cfg.host.as_str(), cfg.port);
    println!("listening on http :// {}: {}", bind_addr.0, bind_addr.1);

    HttpServer::new(move || {
        App::new()
            // share state with handlers as actix_web::web::Data
            .app_data(actix_web::web::Data::from(shared_state.clone()))
            .configure(routes::init_routes)
    })
    .bind(bind_addr)?
    .run()
    .await
    .map_err(|e| eyre::eyre!(e))
}
