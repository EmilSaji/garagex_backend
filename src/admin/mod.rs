pub mod models;
pub mod repository;
pub mod handlers;

use actix_web::web;
use crate::auth::AuthMiddleware;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/login", web::post().to(handlers::login))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::default())
                    .route("/garages", web::get().to(handlers::list_garages))
                    .route("/garages/{id}", web::get().to(handlers::get_garage))
            )
    );
}
