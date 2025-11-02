pub mod admin;
pub mod auth;
pub mod garage;
pub mod config;
pub mod health;
pub mod routes;
pub mod state;

use actix_web::middleware::Logger;
use crate::config::Config;
use crate::state::AppState;
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{App, HttpServer};
use eyre::Result;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn run(cfg: Config) -> Result<()> {
    // Connect to DB
    let pool = PgPool::connect(&cfg.database_url)
        .await
        .map_err(|e| eyre::eyre!("DB connect error: {}", e))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| eyre::eyre!("Migrations failed: {}", e))?;

    // Build state
    let state = AppState {
        db: pool,
        // add other shared clients here
    };

    // Wrap state in Arc **once**
    let shared_state = Arc::new(state);

    // Bind address
    let bind_addr = (cfg.host.as_str(), cfg.port);
    println!("listening on http://{}:{}", bind_addr.0, bind_addr.1);

    HttpServer::new(move || {
        // clone Arc for each worker/app instance
        let state_for_app = shared_state.clone();

        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            // **This registers web::Data<std::sync::Arc<AppState>>**
            .app_data(actix_web::web::Data::from(state_for_app))
            .configure(routes::init_routes)
    })
    .bind(bind_addr)?
    .run()
    .await
    .map_err(|e| eyre::eyre!(e))
}
