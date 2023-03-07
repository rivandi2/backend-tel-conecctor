use actix_web::{web, web::Data, HttpResponse};

use crate::{util::client::Klien, models::connector::Connector};

pub async fn post(klien: Data<Klien>, payload: web::Json<Connector>) -> HttpResponse {
    let get = klien.add_connector(&payload).await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().finish()
    };
}

pub async fn get(klien: Data<Klien>) -> HttpResponse {
    let get = klien.get_connectors().await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().finish()
    };
}

pub async fn get_one(klien: Data<Klien>, id: web::Path<String>) -> HttpResponse {
    let get = klien.get_one_connector(id.to_string()).await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().finish()
    };
}

pub async fn delete(klien: Data<Klien>, id: web::Path<String>) -> HttpResponse {
    let get = klien.delete_connector(id.to_string()).await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().finish()
    };
}

pub async fn update(klien: Data<Klien>, id: web::Path<String>, payload: web::Json<Connector>) -> HttpResponse {
    let get = klien.update_connector(id.to_string(), &payload).await;
    match get {
        Ok(ok) => return HttpResponse::Ok().json(ok),
        Err(e)=> return HttpResponse::InternalServerError().finish()
    };
}


