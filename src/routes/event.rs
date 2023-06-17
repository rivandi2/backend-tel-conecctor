use actix_web::{web, HttpResponse, HttpRequest};
use serde_json::Value;

use crate::actions::event;
use crate::client;

pub async fn post(db: web::Data<client::rusoto::Client>, req: HttpRequest, payload: web::Json<Value>, id: web::Path<String>) -> HttpResponse {
    let event_key = req
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if event_key.starts_with("Atlassian") {
        match event::process_event(&db.s3, payload.into_inner(), id.to_string()).await{
            Ok(o)=>println!("{:?}", o),
            Err(e)=>println!("{:?}", e)
        };
        return HttpResponse::Ok().json("Event Recieved")
    } else {
        println!("{:?}", event_key);
        return HttpResponse::BadRequest().json("Not from Jira")
    }
    
}

