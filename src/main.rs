use actix_web::{web, web::{Data, resource}, App, HttpServer, middleware::Logger};

mod routes;
mod util;
mod models;
mod errortype;
mod actions;
mod client;
extern crate serde_json;

use routes::{jira, connector, event};
use dotenv::dotenv;
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    let bind_ip = std::env::var("BIND_IP").expect("BIND_IP must be defined");
    let bind_port = std::env::var("BIND_PORT").expect("BIND_PORT must be defined").parse::<u16>().unwrap_or(8082);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(Data::new(client::rusoto::Client::new()))
            .app_data(Data::new(client::jira::Client::new()))
            .service(resource("/projects").route(web::get().to(jira::get)))
            .service( web::scope("/connector")
                .route("", web::post().to(connector::post))
                .route("", web::get().to(connector::get))
                .route("{name}", web::get().to(connector::get_one))
                .route("{name}", web::delete().to(connector::delete))
                .route("{name}", web::put().to(connector::update))
                )
            .service(resource("/event/telkomdevelopernetwork").route(web::post().to(event::post)))    
    })
    .bind((bind_ip, bind_port))?
    .run()
    .await
}