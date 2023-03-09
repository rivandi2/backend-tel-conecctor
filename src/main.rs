use actix_web::{web, web::{Data, resource}, App, HttpServer, middleware::Logger};

mod routes;
mod util;
mod models;
extern crate serde_json;

use routes::{jira, connector};
use util::client::Klien;
use dotenv::dotenv;
use actix_cors::Cors;


#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let klien = Klien::new();
    
    actix_rt::spawn(async move {
        loop {
            let duration = std::time::Duration::from_secs(60);
            let _tes = klien.get_hookdeck_events_filter().await;
            match _tes {
                Ok(events) => println!("{:?}", events),
                Err(e)=> println!("{:?}", e)
            };
            tokio::time::sleep(duration).await;
        }
    });
   
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(Data::new(Klien::new()))
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