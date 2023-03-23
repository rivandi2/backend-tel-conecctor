use actix_web::{web, web::Data, HttpResponse};

use crate::{util::client::Klien, models::connector::Connector};

pub async fn post(klien: Data<Klien>, payload: web::Json<Connector>) -> HttpResponse {
    let get = klien.add_connector(&payload).await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().json(format!("{}",e))
    };
}

pub async fn get(klien: Data<Klien>) -> HttpResponse {
    let get = klien.get_connectors().await;
    match get {
        Some(ok) => return HttpResponse::Ok().json(ok),
        None => return HttpResponse::InternalServerError().json("Connector list empty")
    };
}

pub async fn get_one(klien: Data<Klien>, name: web::Path<String>) -> HttpResponse {
    let get = klien.get_one_connector(name.to_string()).await;
    match get {
        Some(ok) => return HttpResponse::Ok().json(ok),
        None => return HttpResponse::NotFound().json(format!("No connector with name: {} found", name))
    };
}

pub async fn delete(klien: Data<Klien>, name: web::Path<String>) -> HttpResponse {
    let get = klien.delete_connector(name.to_string()).await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().json(format!("{}",e))
    };
}

pub async fn update(klien: Data<Klien>, name: web::Path<String>, payload: web::Json<Connector>) -> HttpResponse {
    let get = klien.update_connector(name.to_string(), &payload).await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().json(format!("{}",e))
    };
}


