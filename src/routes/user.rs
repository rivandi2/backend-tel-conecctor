use actix_web::{web::{self, ReqData}, HttpResponse};
use actix_web_httpauth::extractors::basic::BasicAuth;

use crate::{client, models::user::UserInput, middleware, middleware::Claims};

pub async fn register(db: web::Data<client::rusoto::Client>, client: web::Data<client::mongodb::Client>, payload: web::Json<UserInput>) -> HttpResponse {
    match client.create_user(&db.s3, payload.clone()).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
    };
}

pub async fn login(client: web::Data<client::mongodb::Client>, credentials: BasicAuth, secret: web::Data<String>) -> HttpResponse {
    let username = credentials.user_id();
    let password = credentials.password();
    match password {
        Some(pass) => {
            match client.login(username.to_string(), pass.to_string(), &secret).await {
                Ok(users) => return HttpResponse::Ok().json(users),
                Err(e)=> return HttpResponse::BadRequest().json(e)
            };
        },
        None => return HttpResponse::Unauthorized().json("Username and password required"),
    }
    
}

pub async fn get(client: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &client).await {
        Ok(user) => {
            return HttpResponse::Ok().json(user)
        },
        Err(error) => return error
    }  
}

pub async fn delete(db: web::Data<client::rusoto::Client>, jira: web::Data<client::jira::Client> ,client: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &client).await {
        Ok(user) => {
            match client.delete_user(&db.s3, &jira.reqwest, user).await {
                Ok(ok) => return HttpResponse::Ok().json(ok),
                Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}

