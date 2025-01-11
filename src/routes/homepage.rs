use actix_web::{web, HttpResponse, Responder};
use tera::Tera;

pub async fn homepage(tmpl: web::Data<Tera>) -> impl Responder {
    let rendered = tmpl.render("index.html", &tera::Context::new());

    match rendered {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::InternalServerError().body("Error rendering template"),
    }
}
