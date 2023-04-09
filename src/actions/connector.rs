
use rusoto_s3::{S3, S3Client, GetObjectRequest, ListObjectsV2Request, PutObjectRequest, HeadObjectRequest, DeleteObjectRequest};
use teloxide::prelude::*;
use std::io::Read;

use crate::errortype::ConnectorError;
use crate::models::connector::Connector;
use crate::actions::log;

const BUCKET: &'static str = "atlassian-connector";
const FOLDER: &'static str = "Connectors";
const LOGS: &'static str = "logs";

pub async fn add_connector(db: &S3Client, payload: &Connector) -> Result<String, ConnectorError> {
    if connector_exist(&db, &payload.name).await {
        return Err(ConnectorError::ConCreateExist)
    } else {
        match db.put_object(PutObjectRequest {
            bucket: BUCKET.to_owned(),
            key: format!("{}/{}.yml", FOLDER.to_owned(), payload.name),
            body: Some((serde_yaml::to_string(&payload).unwrap().into_bytes()).into()),
            ..Default::default()
        }).await{
            Ok(_) => {
                match log::add_log(&db, payload.name.clone(), None).await{
                    Ok(_) => return Ok("Connector successfuly created".to_owned()),
                    Err (e) => return Err(ConnectorError::RusError(e.to_string()))
                }
                // return Ok("Connector successfuly created".to_owned())
            },
            Err(e) => return Err(ConnectorError::RusError(e.to_string()))
        }
    }
}

pub async fn delete_connector(db: &S3Client, target_name: String) -> Result<String, ConnectorError> {
    if connector_exist(&db, &target_name).await {
        match db.delete_object( DeleteObjectRequest  {
            bucket: BUCKET.to_owned(),
            key: format!("{}/{}.yml", FOLDER.to_owned(), target_name),
            ..Default::default()
        }).await{
            Ok(_) => {
                let _res = db.delete_object( DeleteObjectRequest  {
                    bucket: BUCKET.to_owned(),
                    key: format!("{}/{}.csv", LOGS.to_owned(), target_name),
                    ..Default::default()
                }).await;
                return Ok("Connector successfuly deleted".to_owned())
            },
            Err(e) => return Err(ConnectorError::RusError(e.to_string()))
        }
    } else {
        return Err(ConnectorError::ConNotFound)
    }
}

pub async fn update_connector(db: &S3Client, target_name: String, payload: &Connector) -> Result<String, ConnectorError> {
    if connector_exist(&db, &target_name).await {
        if connector_exist(&db, &payload.name).await && &payload.name != &target_name {
            return Err(ConnectorError::ConUpdateExist)
        } 
    
        if payload.bot_type.to_lowercase() == "telegram".to_owned() { 
            let bot = teloxide::Bot::new(&payload.token.to_owned());

            if let Err(_e) = bot.get_me().await {
                return Err(ConnectorError::TokenInval)
            }

            if let Err(_e) = bot.get_chat(payload.chatid.clone()).send().await {
                return Err(ConnectorError::ChatidInval)
            }
        } else {
            match slack_hook2::Slack::new(&payload.token) {
                Ok(s)=> {
                    if let Err(_) = s.send(&slack_hook2::PayloadBuilder::new()
                        .text("This is a connection test message")
                        .build()
                        .unwrap()).await {
                        return Err(ConnectorError::TokenInval)
                    }
                },
                Err(_) => return Err(ConnectorError::TokenInval)
            }
        };

        match db.put_object(PutObjectRequest {
            bucket: BUCKET.to_string(),
            key: format!("{}/{}.yml", FOLDER.to_owned(), payload.name),
            body: Some((serde_yaml::to_string(&payload).unwrap().into_bytes()).into()),
            ..Default::default()
        }).await{
            Ok(_) => {          
                if &payload.name != &target_name {
                    match db.delete_object(DeleteObjectRequest  {
                        bucket: BUCKET.to_owned(),
                        key: format!("{}/{}.yml", FOLDER.to_owned(), target_name),
                        ..Default::default()
                    }).await {
                        Ok(_) =>  {
                            log::add_log(&db, payload.name.clone(), Some(log::get_one_log(&db, target_name.clone()).await.unwrap())).await;
                            db.delete_object(DeleteObjectRequest  {
                                bucket: BUCKET.to_owned(),
                                key: format!("{}/{}.csv", LOGS.to_owned(), target_name),
                                ..Default::default()
                            }).await;

                            return Ok("Connector successfuly updated! ps.new name".to_owned())
                        },
                        Err(e) => return Err(ConnectorError::RusError(e.to_string()))
                    }
                } else {
                    return Ok("Connector successfuly updated!".to_owned())
                }
            }, 
            Err(e) => return Err(ConnectorError::RusError(e.to_string()))
        }
    } else {
        return Err(ConnectorError::ConNotFound)
    }
}

pub async fn get_connectors(db: &S3Client) -> Result<Vec<Connector>, ConnectorError> {
    match db.list_objects_v2(ListObjectsV2Request {
        bucket: BUCKET.to_owned(),
        prefix: Some(format!("{}/", FOLDER.to_owned())),
        ..Default::default()
    }).await {
        Ok(object) => {
            if &object.contents.as_ref().unwrap().len() == &"1".parse::<usize>().unwrap() {
                return Err(ConnectorError::ConEmpty)
            } else {
                let list: Vec<String> = object.contents.unwrap()
                    .into_iter()
                    .flat_map(|ob| {
                        if ob.key.as_ref().unwrap().ends_with(".yml") { Some(ob.key.unwrap())} 
                        else { None }
                    })
                    .collect();
                
                let mut cons: Vec<Connector> = Vec::new();

                for pa in list {
                    match db.get_object(GetObjectRequest {
                        bucket: BUCKET.to_owned(),
                        key: pa.clone(),
                        ..Default::default()
                    }).await {
                        Ok(ob) =>{
                            let result = tokio::task::spawn_blocking(|| {
                                let mut data = String::new();
                                ob.body.unwrap().into_blocking_read().read_to_string(&mut data);
                                let yaml: Connector = serde_yaml::from_str(&data).unwrap();
                                return yaml
                            }).await.expect("Task panicked");
                            cons.push(result);
                        },
                        Err(e) => return Err(ConnectorError::RusError(e.to_string()))
                    }    
                }   
                return Ok(cons)
            }
        },
        Err(e) => return Err(ConnectorError::RusError(e.to_string()))
    }
    
}

#[allow(warnings)]
pub async fn get_one_connector(db: &S3Client, target_name: String) -> Option<Connector> {
    if connector_exist(&db, &target_name).await {
        match db.get_object(GetObjectRequest {
            bucket: BUCKET.to_owned(),
            key: format!("{}/{}.yml", FOLDER.to_owned(), target_name),
            ..Default::default()
        }).await {
            Ok(ob) =>{
                let result = tokio::task::spawn_blocking(|| {
                    let mut data = String::new();
                    ob.body.unwrap().into_blocking_read().read_to_string(&mut data);
                    let yaml: Connector = serde_yaml::from_str(&data).unwrap();
                    return yaml
                }).await.expect("Task panicked");
                Some(result)
            },
            Err(_e) => None
        }
    } else {
        None
    }
}

pub async fn connector_exist(db: &S3Client, name: &str) -> bool {
    match db.head_object(HeadObjectRequest {
        bucket: BUCKET.to_string(),
        key: format!("{}/{}.yml", FOLDER.to_owned(), name.to_owned()),
        ..Default::default()
    }).await{
        Ok(_) => true,
        Err(_) => false
    }
}

