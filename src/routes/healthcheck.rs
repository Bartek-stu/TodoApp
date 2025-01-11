use actix_web::{HttpResponse, Responder};

#[tracing::instrument(name = "Check application health")]
pub async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().finish()
}
