use actix_web::{web, web::{Data, resource}, App, HttpServer, middleware::Logger};

mod routes;
mod util;
mod models;
mod errortype;
extern crate serde_json;

use routes::{jira, connector};
use util::client::Client;
use dotenv::dotenv;
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let client = Client::new();
    
    actix_rt::spawn(async move {
        loop {
            match client.get_hookdeck_events_filter().await {
                Ok(events) => println!("{:?}", events),
                Err(e)=> println!("{:?}", e)
            };
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
    
    let bind_ip = std::env::var("BIND_IP").expect("BIND_IP must be defined");
    let bind_port = std::env::var("BIND_PORT").expect("BIND_PORT must be defined").parse::<u16>().unwrap_or(8082);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(Data::new(Client::new()))
            .service(resource("/projects").route(web::get().to(jira::get)))
            .service( web::scope("/connector")
                .route("", web::post().to(connector::post))
                .route("", web::get().to(connector::get))
                .route("{name}", web::get().to(connector::get_one))
                .route("{name}", web::delete().to(connector::delete))
                .route("{name}", web::put().to(connector::update))
                )
    })
    .bind((bind_ip, bind_port))?
    .run()
    .await
}