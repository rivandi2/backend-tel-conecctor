
use actix_web::{ web, web::Data, HttpResponse};
use serde::{Serialize, Deserialize};

use crate::util::client::Klien;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cred{
    pub email: String,
    pub api_key: String,
}

pub async fn get(client: Data<Klien>, credential: web::Query<Cred>) -> HttpResponse {

    let get = client.get_projects(&credential.email, &credential.api_key).await;
    match get {
        Ok(projects) => return HttpResponse::Ok().json(projects),
        Err(e)=> return HttpResponse::BadRequest().json(format!("{}",e))
    };
}
