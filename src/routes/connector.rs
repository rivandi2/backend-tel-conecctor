use actix_web::{web::{self, ReqData}, HttpResponse};

use crate::{client, actions, models::{connector::{ConnectorInput, Connector}}, errortype::ConnectorError, middleware, middleware::Claims};

pub async fn post(db: web::Data<client::rusoto::Client>, mongodb: web::Data<client::mongodb::Client>, payload: web::Json<ConnectorInput>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            if user.webhook_url.is_none(){
                return HttpResponse::BadRequest().json("You must set up a webhook first")
            }
            if payload.name.is_empty() {
                return HttpResponse::BadRequest().json("Connector name must not be empty!")
            }
            match actions::connector::add_connector(&db.s3, payload.into_inner(), user.id.to_hex()).await {
                Ok(ok) => return HttpResponse::Ok().json(ok),
                Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
                Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}

pub async fn get(db: web::Data<client::rusoto::Client>, mongodb: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>) -> HttpResponse { 
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            match actions::connector::get_connectors(&db.s3, user.id.to_hex()).await {
                Ok(ok) => return HttpResponse::Ok().json(ok),
                Err(_)=> return HttpResponse::NotFound().json("Connector list empty")
            }
        },
        Err(error) => return error
    }   
}

pub async fn get_one(db: web::Data<client::rusoto::Client>, mongodb: web::Data<client::mongodb::Client>, name: web::Path<String>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            match actions::connector::get_one_connector(&db.s3, name.to_string(), user.id.to_hex()).await {
                Some(ok) => return HttpResponse::Ok().json(ok),
                None => return HttpResponse::NotFound().json(format!("No connector with name: {} found", name))
            };
        },
        Err(error) => return error
    }   
}

pub async fn delete(db: web::Data<client::rusoto::Client>, mongodb: web::Data<client::mongodb::Client>, name: web::Path<String>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            match actions::connector::delete_connector(&db.s3, name.to_string(), user.id.to_hex()).await {
                Ok(ok) => return HttpResponse::Ok().json(ok),
                Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
                Err(e) => return HttpResponse::NotFound().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}

pub async fn update(db: web::Data<client::rusoto::Client>, mongodb: web::Data<client::mongodb::Client>, name: web::Path<String>, payload: web::Json<Connector>, req_user: Option<ReqData<Claims>>) -> HttpResponse {
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            if payload.name.is_empty() {
                return HttpResponse::BadRequest().json("Connector name must not be empty!")
            }
            let mut connector = payload.into_inner();
            match actions::connector::update_connector(&db.s3, name.to_string(), &mut connector, user.id.to_hex()).await {
                Ok(ok) => return HttpResponse::Ok().json(ok),
                Err(ConnectorError::ConNotFound) => return HttpResponse::NotFound().json(format!("{}", ConnectorError::ConNotFound)),
                Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
                Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
            };
        },
        Err(error) => return error
    }   
}


