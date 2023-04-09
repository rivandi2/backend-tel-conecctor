use actix_web::{web, HttpResponse};

use crate::{client, actions, models::connector::Connector, errortype::ConnectorError};

pub async fn post(db: web::Data<client::rusoto::Client>, payload: web::Json<Connector>) -> HttpResponse {
    if payload.name.is_empty() {
        return HttpResponse::BadRequest().json("Connector name must not be empty!")
    }
    match actions::connector::add_connector(&db.s3, &payload).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
        Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
    };
}

pub async fn get(db: web::Data<client::rusoto::Client>) -> HttpResponse { 
    match actions::connector::get_connectors(&db.s3).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(_)=> return HttpResponse::NotFound().json("Connector list empty")
    };
}

pub async fn get_one(db: web::Data<client::rusoto::Client>, name: web::Path<String>) -> HttpResponse {
    match actions::connector::get_one_connector(&db.s3, name.to_string()).await {
        Some(ok) => return HttpResponse::Ok().json(ok),
        None => return HttpResponse::NotFound().json(format!("No connector with name: {} found", name))
    };
}

pub async fn delete(db: web::Data<client::rusoto::Client>, name: web::Path<String>) -> HttpResponse {

    match actions::connector::delete_connector(&db.s3, name.to_string()).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
        Err(e) => return HttpResponse::NotFound().json(format!("{}",e))
    };
}

pub async fn update(db: web::Data<client::rusoto::Client>, name: web::Path<String>, payload: web::Json<Connector>) -> HttpResponse {
    if payload.name.is_empty() {
        return HttpResponse::BadRequest().json("Connector name must not be empty!")
    }
    
    match actions::connector::update_connector(&db.s3, name.to_string(), &payload).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(ConnectorError::ConNotFound) => return HttpResponse::NotFound().json(format!("{}", ConnectorError::ConNotFound)),
        Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
        Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
    };
}


