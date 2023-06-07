use actix_web::{web::{self, ReqData}, HttpResponse};
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::{Serialize, Deserialize};

use crate::{client, models::user::UserInput, middleware, middleware::Claims};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cred{
    pub email: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login{
    pub username: String,
    pub password: String,
}

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


//dev only

#[derive(Debug, Serialize, Deserialize)]
pub struct Code{
    pub pass: String
}


pub async fn get_all(client: web::Data<client::mongodb::Client>, load: web::Json<Code>) -> HttpResponse {
    let code = std::env::var("DEV_CODE").expect("DEV_CODE must be defined");
    
    if load.pass == code {
        match client.get_users().await {
            Ok(users) => {
                if users.is_empty() {
                    return HttpResponse::Ok().json("Users list empty")
                } else {
                    return HttpResponse::Ok().json(users)
                }
            },
            Err(_)=> return HttpResponse::InternalServerError().json("Error from server")
        };
    } else {
        return HttpResponse::Unauthorized().json("Site for developer only");
    }
}

pub async fn delete_by_dev(db: web::Data<client::rusoto::Client>, jira: web::Data<client::jira::Client>, client: web::Data<client::mongodb::Client>, load: web::Json<Code>, id: web::Path<String>) -> HttpResponse {
    let code = std::env::var("DEV_CODE").expect("DEV_CODE must be defined");
    
    if load.pass == code {
        match client.get_one_user(id.to_string()).await {
            Ok(users) => {
                if users.is_empty() {
                    return HttpResponse::BadRequest().json("User not found")
                } else {
                    match client.delete_user(&db.s3, &jira.reqwest, users[0].clone()).await{
                        Ok(o) => return HttpResponse::Ok().json(o),
                        Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
                    }
                }
            },
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
        }
    } else {
        return HttpResponse::Unauthorized().json("Site for developer only");
    }
}
