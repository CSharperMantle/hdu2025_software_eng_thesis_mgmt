use actix_web::{HttpResponse, Responder, get};

#[get("/api/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().finish()
}
