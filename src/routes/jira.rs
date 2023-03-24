
use actix_web::{ web, web::Data, HttpResponse};
use serde::{Serialize, Deserialize};

use crate::util::client::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cred{
    pub email: String,
    pub api_key: String,
}

pub async fn get(client: Data<Client>, credential: web::Query<Cred>) -> HttpResponse {
    match client.get_projects(&credential.email, &credential.api_key).await {
        Ok(projects) => return HttpResponse::Ok().json(projects),
        Err(e)=> return HttpResponse::BadRequest().json(format!("{}",e))
    };
}
