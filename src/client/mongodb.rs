use std::str::FromStr;

use chrono::{Duration, Utc};
use futures::TryStreamExt;
use mongodb::bson::doc;
use bcrypt::{hash, verify};
use rusoto_s3::{S3Client, PutObjectRequest, S3, DeleteObjectRequest, ListObjectsV2Request};

use crate::{models::user::{UserInput, UserNew, User}, middleware::Claims};
use jsonwebtoken::{encode, Header, EncodingKey};

const BUCKET: &'static str = "atlassian-connector";

#[derive(Clone)]
pub struct Client {
    pub mongodb: mongodb::Client
}

impl Client {
    pub fn new() -> Self {
        return Self{
            mongodb: futures::executor::block_on(mongodb::Client::with_uri_str(std::env::var("MONGODB_URI").expect("MONGODB URI must be defined"))).unwrap(),
        }
    }

    pub async fn create_user(&self, db: &S3Client, user: UserInput) -> Result<mongodb::results::InsertOneResult, String>  {
        match self
            .mongodb
            .database("telconnect")
            .collection::<User>("users")
            .find(doc! {
                "username": &user.username
            }, None)
            .await{
                Ok(cursor) => {
                    let doc = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
                    if !doc.is_empty(){
                        return Err("Username already taken".to_string())
                    }
                },
                Err(e) => return Err(e.to_owned().to_string())
            }
        
        match self
            .mongodb
            .database("telconnect")
            .collection::<UserNew>("users")
            .insert_one(UserNew{
                username: user.username,
                password: hash(user.password, 4).unwrap(),
                
                created_at: chrono::Utc::now()
                    .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).unwrap()),
                    // .format("%d/%m/%Y %H:%M").to_string(),
    
                jira_email: None,
                jira_api_key: None,
                jira_url: None,
                webhook_url: None,
                webhook_functional: None,
                webhook_last_check: None
            }, None)
            .await {
                Ok(o) => { 
                    match db.put_object(PutObjectRequest {
                        bucket: BUCKET.to_owned(),
                        key: format!("{}/", o.inserted_id.as_object_id().expect("Failed to get inserted ID").to_hex()),
                        body: None,
                        ..Default::default()
                    }).await {
                        Ok(_) => return Ok(o),
                        Err(e) => return Err(e.to_string())
                    }
                },
                Err(e) => return Err(e.to_string())
            };
    }

    pub async fn login(&self, username: String, password: String, secret: &str) -> Result<String, String> {
        match self
            .mongodb
            .database("telconnect")
            .collection::<User>("users")
            .find(doc! {
                "username": username
            }, None)
            .await {
            Ok(cursor) => {
                let doc = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
                if doc.is_empty(){
                    return Err("User Not Found".to_string())
                }

                match verify(password, &doc[0].password){
                    Ok(ver) => {
                        if ver == true {
                            let claims = Claims { 
                                id: doc[0].id,
                                exp: (Utc::now() + Duration::hours(2)).timestamp() as usize
                            };
                            let token: String = encode(
                                &Header::default(),
                                &claims,
                                &EncodingKey::from_secret(secret.as_ref()),
                            ).unwrap();
                            return Ok(token)
                        } else {
                            return Err("Invalid password".to_string())
                        }
                    },
                    Err(e)=> println!("{}", e.to_string())
                }
                
                return Err("Password incorrect".to_string())
            },
            Err(e) => return Err(e.to_owned().to_string())
        }
    }
    
    pub async fn get_one_user(&self, id: String) -> Result<Vec<User>, mongodb::error::Error> {
        match self
            .mongodb
            .database("telconnect")
            .collection::<User>("users")
            .find(doc! {
                "_id": mongodb::bson::oid::ObjectId::from_str(&id).unwrap()
            }, None)
            .await {
            Ok(cursor) => {
                let doc = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
                return Ok(doc)
            }
            Err(e) => return Err(e)
        }
    }

    pub async fn delete_user(&self, db: &S3Client, jira: &reqwest::Client, user: User) -> Result<mongodb::results::DeleteResult, String> {
        if user.webhook_url.is_some(){
            let _del = jira
                .delete(user.webhook_url.unwrap())
                .basic_auth(user.jira_email.unwrap().to_string(), Some(user.jira_api_key.unwrap().to_string()))
                .send()
                .await;
        }
        
        match self
            .mongodb
            .database("telconnect")
            .collection::<User>("users")
            .delete_one(
                doc! {
                    "_id": mongodb::bson::oid::ObjectId::from_str(&user.id.to_hex()).unwrap()
                }, None)
            .await {
            Ok(o) => {
                match db.list_objects_v2(ListObjectsV2Request {
                    bucket: BUCKET.to_owned(),
                    prefix: Some(format!("{}/", user.id.to_hex())),
                    ..Default::default()
                }).await{
                    Ok(objects) => {
                        let list: Vec<String> = objects.contents.unwrap()
                            .into_iter()
                            .rev()
                            .map(|ob| ob.key.unwrap())
                            .collect();
                        for li in list {
                            db.delete_object( DeleteObjectRequest  {
                                bucket: BUCKET.to_owned(),
                                key: li.clone(),
                                ..Default::default()
                            }).await;
                        }
                        return Ok(o)
                    },
                    Err(e) => return Err(e.to_string())
                }
            },
            Err(e) => return Err(e.to_string())
        }
    }

    /// development only ///
    pub async fn get_users(&self) -> Result<Vec<User>, mongodb::error::Error> {
        match self
            .mongodb
            .database("telconnect")
            .collection::<User>("users")
            .find(None, None)
            .await {
            Ok(cursor) => {
                let doc = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
                return Ok(doc)
            }
            Err(e) => return Err(e)
        }
    }

}


