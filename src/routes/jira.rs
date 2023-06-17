
use actix_web::{ web::{self, ReqData}, web::Data, HttpResponse};
use serde::{Serialize, Deserialize};

use crate::{client, middleware, middleware::Claims};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookInput{
    pub email: String,
    pub api_key: String,
    pub jira_url: String
}

pub async fn get_project(client: Data<client::jira::Client>, mongodb: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            if user.webhook_url.is_none() {
                return HttpResponse::BadRequest().json("You must set up a webhook first")
            }
            match client.get_projects(user).await {
                Ok(projects) => return HttpResponse::Ok().json(projects),
                Err(e)=> return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}

pub async fn post_webhook(client: Data<client::jira::Client>, mongodb: web::Data<client::mongodb::Client>, webhook: web::Json<WebhookInput>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            if user.webhook_url.is_some(){
                return HttpResponse::BadRequest().json("You have already created a webhook for this account")
            }

            match client.create_webhook(&mongodb.mongodb, &webhook, user).await {
                Ok(message) => return HttpResponse::Ok().json(message),
                Err(e)=> return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}

pub async fn delete_webhook(client: Data<client::jira::Client>, mongodb: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            if user.webhook_url.is_none(){
                return HttpResponse::BadRequest().json("You haven't created a webhook")
            } 

            match client.delete_webhook(&mongodb.mongodb, user).await {
                Ok(message) => return HttpResponse::Ok().json(message),
                Err(e)=> return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}


pub async fn check_webhook(client: Data<client::jira::Client>, mongodb: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            if user.webhook_url.is_none(){
                return HttpResponse::BadRequest().json("You haven't created a webhook")
            } 
            match client.check_webhook(&mongodb.mongodb, user).await {
                Ok(message) => return HttpResponse::Ok().json(message),
                Err(e)=> return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}

pub async fn put_webhook(client: Data<client::jira::Client>, mongodb: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            if user.webhook_url.is_none(){
                return HttpResponse::BadRequest().json("You haven't created a webhook")
            } 

            if user.webhook_functional.is_none() {
                return HttpResponse::BadRequest().json("Please check your webhook status at least once before repairing")
            }
            
            if user.webhook_functional.is_some() && user.webhook_functional.unwrap() == true {
                return HttpResponse::BadRequest().json("Your current webhook status is functional, please check webhook status again beforehand if you think your webhook is non functional")
            } 

            match client.repair_webhook(&mongodb.mongodb, user).await {
                Ok(message) => return HttpResponse::Ok().json(message),
                Err(e)=> return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}
