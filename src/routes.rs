use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(crate::health::health_handler))
            .configure(crate::admin::init_routes) // no semicolon here
    );
}
