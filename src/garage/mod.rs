pub mod handlers;
pub mod models;
pub mod repository;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/garage")
            .route("/login", web::post().to(handlers::login))
            .route("/users/{user_id}/jobs", web::get().to(handlers::list_jobs_for_user))
            .route("/users/{user_id}/jobs", web::post().to(handlers::create_job_for_user))
            .route("/jobs/{job_id}", web::get().to(handlers::get_job_details))
            .route("/jobs/{job_id}/status", web::post().to(handlers::update_job_status))
            .route("/jobs/{job_id}/parts", web::post().to(handlers::add_job_parts))
            .route("/jobs/{job_id}/parts/{part_id}", web::post().to(handlers::update_job_part))
            .route("/jobs/{job_id}/parts/{part_id}", web::delete().to(handlers::delete_job_part)),
    );
}
