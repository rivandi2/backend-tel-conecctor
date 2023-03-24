use actix_web::{web, web::Data, HttpResponse};

use crate::{util::client::Client, models::connector::Connector, errortype::ConnectorError};

pub async fn post(client: Data<Client>, payload: web::Json<Connector>) -> HttpResponse {
    if payload.name.is_empty() {
        return HttpResponse::BadRequest().json("Connector name must not be empty!")
    }

    match client.add_connector(&payload).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
        Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
    };
}

pub async fn get(client: Data<Client>) -> HttpResponse {
    match client.get_connectors().await {
        Some(ok) => return HttpResponse::Ok().json(ok),
        None => return HttpResponse::NotFound().json("Connector list empty")
    };
}

pub async fn get_one(client: Data<Client>, name: web::Path<String>) -> HttpResponse {
    match client.get_one_connector(name.to_string()).await {
        Some(ok) => return HttpResponse::Ok().json(ok),
        None => return HttpResponse::NotFound().json(format!("No connector with name: {} found", name))
    };
}

pub async fn delete(client: Data<Client>, name: web::Path<String>) -> HttpResponse {
    match client.delete_connector(name.to_string()).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
        Err(e) => return HttpResponse::NotFound().json(format!("{}",e))
    };
}

pub async fn update(client: Data<Client>, name: web::Path<String>, payload: web::Json<Connector>) -> HttpResponse {
    if payload.name.is_empty() {
        return HttpResponse::BadRequest().json("Connector name must not be empty!")
    }
    
    match client.update_connector(name.to_string(), &payload).await {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(ConnectorError::ConNotFound) => return HttpResponse::NotFound().json(format!("{}", ConnectorError::ConNotFound)),
        Err(ConnectorError::RusError(e)) => return HttpResponse::InternalServerError().json(format!("{}", e)),
        Err(e) => return HttpResponse::BadRequest().json(format!("{}",e))
    };
}


