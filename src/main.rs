use actix_web::{get, post, web, web::{Data, resource}, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};

mod routes;
mod util;
mod models;
extern crate serde_json;

use routes::{event, jira, connector};
use util::client::Klien;
use dotenv::dotenv;

async fn manual() -> impl Responder {
    HttpResponse::Ok().body("Heyo")
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let klien = Klien::new();
    
    actix_rt::spawn(async move {
        loop {
            let duration = std::time::Duration::from_secs(60);
            let _tes = klien.get_hookdeck_events().await;
            match _tes {
                Ok(events) => println!("{:?}", events),
                Err(e)=> println!("{:?}", e)
            };
            tokio::time::sleep(duration).await;
        }
    });
   
    // .service(resource("/events").route(web::get().to(event::get)))
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(Klien::new()))
            .route("/hey", web::get().to(manual))
            .service(resource("/projects").route(web::get().to(jira::get)))
            .service( web::scope("/connector")
                .route("", web::post().to(connector::post))
                .route("", web::get().to(connector::get))
                .route("{id}", web::get().to(connector::get_one))
                .route("{id}", web::delete().to(connector::delete))
                .route("{id}", web::put().to(connector::update))
                )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}