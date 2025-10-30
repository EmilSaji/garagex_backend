use actix_web::{HttpResponse, Responder};

pub async fn health_handler() -> impl Responder {
    HttpResponse::Ok().body("ok")
}
