use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};

async fn manual() -> impl Responder {
    HttpResponse::Ok().body("Heyo")
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    HttpServer::new(|| {
        App::new()
            .route("/hey", web::get().to(manual))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}