use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(crate::health::health_handler))
        // .configure(crate::auth::init_routes)
        // .configure(crate::garages::init_routes)
        // .configure(crate::jobs::init_routes)
        // add other domain route configs here
    );
}
