use actix_web::{web, web::{Data, resource}, App, HttpServer, middleware::Logger};
use actix_web_httpauth::middleware::HttpAuthentication;

mod routes;
mod models;
mod errortype;
mod actions;
mod client;
mod middleware;
extern crate serde_json;

use routes::{jira, connector, event, user, log};
use dotenv::dotenv;
use actix_cors::Cors;
use middleware::validator;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be defined");
    
    let bind_ip = std::env::var("BIND_IP").expect("BIND_IP must be defined");
    let bind_port = std::env::var("BIND_PORT").expect("BIND_PORT must be defined").parse::<u16>().unwrap_or(8082);

    HttpServer::new(move || {
        let middleware = HttpAuthentication::bearer(validator);
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(Data::new(client::rusoto::Client::new()))
            .app_data(Data::new(client::jira::Client::new()))
            .app_data(Data::new(client::mongodb::Client::new()))
            .app_data(Data::new(String::from(&jwt_secret)))
            .service(resource("/register").route(web::post().to(user::register)))
            .service(resource("/login").route(web::get().to(user::login)))

            .service( web::scope("/event")
                .route("{id}", web::post().to(event::post))
            )

            .service( web::scope("")
                .wrap(middleware)
                .service( web::scope("/connector")
                    .route("", web::post().to(connector::post))
                    .route("", web::get().to(connector::get))
                    .route("{name}", web::get().to(connector::get_one))
                    .route("{name}", web::delete().to(connector::delete))
                    .route("{name}", web::put().to(connector::update))
                )
                .service( web::scope("/log")
                    .route("{name}", web::get().to(log::get))
                )
                .service(resource("/projects").route(web::get().to(jira::get_project)))
                .service(web::scope("/webhook")
                    .route("", web::post().to(jira::post_webhook))
                    .route("", web::delete().to(jira::delete_webhook))
                    .route("", web::get().to(jira::check_webhook))
                    .route("/repair", web::get().to(jira::put_webhook))
                )
                .service( web::scope("/user")
                    .route("", web::get().to(user::get))
                    .route("", web::delete().to(user::delete))
                )
            )
            

    })
    .bind((bind_ip, bind_port))?
    .run()
    .await
}