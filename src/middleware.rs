use actix_web::{dev::ServiceRequest, error::Error, HttpMessage, HttpResponse, web::{ReqData, self}};
use actix_web_httpauth::{extractors::{  
    bearer::{self, BearerAuth},
    AuthenticationError,}
};

use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey, TokenData, errors::Error as JwtError};
use serde::{Serialize, Deserialize};

use crate::{client, models::user::User};

#[derive(Debug, Serialize, Deserialize, Clone)] 
pub struct Claims{
    pub id: mongodb::bson::oid::ObjectId,
    pub exp: usize
}

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token_string = credentials.token();
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be defined");

    let decoded: Result<TokenData<Claims>, JwtError> = decode::<Claims>(
        &token_string,
        &DecodingKey::from_secret(secret.as_str().as_ref()),
        &Validation::new(Algorithm::HS256),
    );
    match decoded {
        Ok(token) => {
            req.extensions_mut().insert(token.claims);
            Ok(req)
        }
        Err(_) => {
            let config = req
                .app_data::<bearer::Config>()
                .cloned()
                .unwrap_or_default()
                .scope("");
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

pub async fn validate(req_user: Option<ReqData<Claims>>, mongodb: &web::Data<client::mongodb::Client>) -> Result<User, actix_web::HttpResponse> {
    match req_user {
        Some(user) => {
            match mongodb.get_one_user(user.id.to_hex()).await {
                Ok(users) => {
                    if users.is_empty() {
                        return Err(HttpResponse::Unauthorized().json("User already deleted"))
                    } else {
                        return Ok(users[0].clone())
                    }
                },
                Err(e) => return Err(HttpResponse::InternalServerError().json(e.to_string()))
            }
        }, 
        None => return Err(HttpResponse::Unauthorized().json("Must be logged in"))
    }    
}