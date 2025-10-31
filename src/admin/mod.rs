pub mod handlers;
pub mod models;
pub mod repository;

use crate::auth::AuthMiddleware;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/login", web::post().to(handlers::login))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::default())
                    .route("/garages", web::get().to(handlers::list_garages))
                    .route("/garages", web::post().to(handlers::add_garage))
                    .route("/garages/{id}", web::delete().to(handlers::delete_garage))
                    .route("/garages/{id}", web::get().to(handlers::get_garage)),
            ),
    );
}
