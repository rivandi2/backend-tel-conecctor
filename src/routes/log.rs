use actix_web::{ web::{self, ReqData}, HttpResponse};

use crate::{client, actions, middleware, middleware::Claims};

pub async fn get(db: web::Data<client::rusoto::Client>, mongodb: web::Data<client::mongodb::Client>, req_user: Option<ReqData<Claims>>, name: web::Path<String>) -> HttpResponse { 
    match middleware::validate(req_user, &mongodb).await {
        Ok(user) => {
            match actions::log::get_one_log(&db.s3, name.to_string(), user.id.to_hex()).await {
                Ok(log) => return HttpResponse::Ok().json(log),
                Err(_)=> return HttpResponse::NotFound().json("Log not found")
            }
        },
        Err(error) => return error
    }   
}