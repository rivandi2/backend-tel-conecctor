use actix_web::{ web::Data, HttpResponse};

use crate::util::client::Klien;

pub async fn get(klien: Data<Klien>) -> HttpResponse {

    let get = klien.get_hookdeck_events().await;
    match get {
        Ok(events) => return HttpResponse::Ok().json(events),
        Err(e)=> return HttpResponse::BadRequest().finish()
    };
}
