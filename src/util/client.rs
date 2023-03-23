
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

use std::env;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};
use mongodb::bson::{Document, doc};
use mongodb::options::FindOptions;
use futures::TryStreamExt;
use teloxide::prelude::*;
use serde_json::Value;
use slack_hook2::{Slack, PayloadBuilder};
use std::io::Read;
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client, GetObjectRequest, ListObjectsV2Request, PutObjectRequest, HeadObjectRequest, DeleteObjectRequest};
use rusoto_credential::{AwsCredentials, StaticProvider};
use derivative::Derivative;

use crate::models::{jira::{ProjectList, SaringProject},
            event::HookdeckEvents,
            connector::Connector };
use crate::errortype::{JiraError, ConnectorError};

const BUCKET: &'static str = "atlassian-connector";
const FOLDER: &'static str = "Connectors";

#[derive(Debug, Serialize, Deserialize)]
pub struct LastCreated{
    pub created_at: String
}

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Klien 
{
    pub reqwest: reqwest::Client,
    pub mongodb: mongodb::Client,

    #[derivative(Debug="ignore")]
    pub s3: S3Client
}

impl Klien
{
    pub fn new() -> Self
    {
        let reqwest= reqwest::Client::new();
        let mongodb= futures::executor::block_on(mongodb::Client::with_uri_str("mongodb://rivandi:Z0BDxmHQc9k237ES@ac-xtrdikx-shard-00-00.kixsqjq.mongodb.net:27017,ac-xtrdikx-shard-00-01.kixsqjq.mongodb.net:27017,ac-xtrdikx-shard-00-02.kixsqjq.mongodb.net:27017/?ssl=true&replicaSet=atlas-fnfn1t-shard-0&authSource=admin&retryWrites=true&w=majority")).unwrap();

        let creds = AwsCredentials::new("DO00HWJRZGDYX7WLQ63C".to_owned(), "XJRjx4vkpKTL36fkrCOVsOG5fjJtVbo59FMwA3IDCww".to_owned(), None, None);
        let provider = StaticProvider::from(creds);
    
        let s3 = S3Client::new_with(
            rusoto_core::request::HttpClient::new().unwrap(),
            provider,
            Region::Custom {
                name: "sgp1".to_owned(),
                endpoint: "https://sgp1.digitaloceanspaces.com".to_owned(),
            },
        );

        return Self{
            reqwest,
            mongodb, 
            s3
        }
    }

    pub async fn get_projects(&self, email: &str, key: &str) -> Result<Vec<SaringProject>, JiraError>{
        match self.get_request(
            email.to_owned(),
            key.to_owned(),
            "https://telkomdevelopernetwork.atlassian.net/rest/api/3/project".to_owned())
            .await {
            Ok(text) => {
                    if text.contains("Basic authentication with passwords is deprecated") {
                        return Err(JiraError::ApiKeyError)
                    } 
                    let list: Vec<ProjectList> = serde_json::from_str(&text).unwrap();
                    if list.len() == 0 {return Err(JiraError::EmailError)};

                    let saring: Vec<SaringProject> = list.into_iter()
                        .map(|po| SaringProject { id: po.id, name: po.name })
                        .collect();
                    return Ok(saring)
            }
            Err (e) => return Err(JiraError::ErrorMessage(e.to_string()))
        }
    }

    pub async fn get_hookdeck_events_filter(&self) -> Result<String, JiraError> {
        let doc = self.mongodb
            .database("telcon")
            .collection::<Document>("lastcreated")
            .find(None, 
                FindOptions::builder().projection(doc! {
                "_id": 0
            }).build()).await
            .unwrap().try_collect().await.unwrap_or_else(|_| vec![]);
        
        let mut url = "https://api.hookdeck.com/2023-01-01/events".to_owned();

        if doc.is_empty(){
            url = format!("{}?limit=50", url); //set limit optional
        } else {
            let tex: Vec<LastCreated>  = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
            url = format!("{}?created_at[gt]={}", url, tex[0].created_at);
        }

        match self.get_request(
            env::var("HOOKDECK_API_KEY").expect("APIKEYNOTFOUND").to_owned(), 
            "".to_owned(), 
            url)
            .await {
            Ok(text) => {
                let list: HookdeckEvents = serde_json::from_str(&text).unwrap();
                if list.models.is_empty(){
                    return Ok("No New Event".to_owned())
                } else {
                    return Ok(self.loop_event_filter(list).await)
                }      
            },
            Err (e) => return Err(JiraError::ErrorMessage(e.to_string()))
        }
    }

    pub async fn loop_event_filter(&self, events: HookdeckEvents)-> String {
        let mut i = 0;
        let col1 = self.mongodb.database("telcon").collection::<Document>("lastcreated");
        let _drp = col1.delete_many(doc!{}, None).await;
        let _tes = col1.insert_one(doc!{"created_at": &events.models[0].created_at}, None).await;
        
        for ev in events.models.iter().rev() {
            match self.get_hookdeck_event_data(&ev.id).await {
                Ok(val) => self.filter_event(val, &ev.created_at).await,
                Err(_e) => println!("Error"),
            };
            i+=1;
        }
        return format!("{} New Event",i)
    }

    pub async fn get_hookdeck_event_data(&self, id: &str) -> Result<Value, JiraError> {
        match self.get_request(
            env::var("HOOKDECK_API_KEY").expect("APIKEYNOTFOUND").to_owned(), 
            "".to_owned(), 
            format!("https://api.hookdeck.com/2023-01-01/events/{}", id))
            .await {
            Ok(text) => {
                    let list: Value = serde_json::from_str(&text).unwrap();
                    return Ok(list)
                    }
            Err (e) => return Err(e)
        }
    }

    pub async fn get_request(&self, username: String, password: String, url: String) -> Result<String, JiraError> {
        let response = self.reqwest
            .get(url)
            .basic_auth(username, Some(password))
            .send()
            .await;
        if response.is_err() {
            return Err(JiraError::RequestFail);
        } else {
            match response.unwrap().text().await {
                Ok(sip) => return Ok(sip),
                Err(_e) => return Err(JiraError::TextChange)
            }
        }
    }

    pub async fn filter_event(&self, val: Value, time: &str) {
        let source = val.get("data")
            .and_then(|data| {
                data.get("headers").and_then(|headers| {
                    headers.get("user-agent").and_then(|v| v.as_str())
                })
            }).unwrap_or("");
            
        if source == "Atlassian Webhook HTTP Client" {
            let webhook_event= val.get("data")
                .and_then(|data| {
                    data.get("body").and_then(|body| {
                        body.get("webhookEvent").and_then(|v| v.as_str().map(String::from))
                    })
                }).unwrap_or_else(|| "".to_string());

            let project_id = val.get("data")
                .and_then(|data| {
                    data.get("body").and_then(|body| {
                        body.get("issue").and_then(|issue|{
                            issue.get("fields").and_then(|fields|{
                                fields.get("project").and_then(|project|{
                                    project.get("id").and_then(|v| v.as_str().map(String::from))
                                })
                            })    
                        })        
                    })
                }).unwrap_or_else(|| "".to_string());

            let project_name = val.get("data")
                .and_then(|data| {
                    data.get("body").and_then(|body| {
                        body.get("issue").and_then(|issue|{
                            issue.get("fields").and_then(|fields|{
                                fields.get("project").and_then(|project|{
                                    project.get("name").and_then(|v| v.as_str().map(String::from))
                                })
                            })    
                        })        
                    })
                }).unwrap_or_else(|| "".to_string());    

            let mut user = "u".to_string();

            if webhook_event.contains("issue") {
                user = val.get("data")
                    .and_then(|data| {
                        data.get("body").and_then(|body| {
                            body.get("user").and_then(|user|{
                                user.get("displayName").and_then(|v| v.as_str().map(String::from)) 
                            })        
                        })
                    }).unwrap_or_else(|| "".to_string());
            } else if webhook_event.contains("comment") {
                user = val.get("data")
                    .and_then(|data| {
                        data.get("body").and_then(|body| {
                            body.get("comment").and_then(|comment|{
                                comment.get("author").and_then(|author|{
                                    author.get("displayName").and_then(|v| v.as_str().map(String::from)) 
                                })    
                            })        
                        })
                    }).unwrap_or_else(|| "".to_string());
            }

            match self.find_connectors(&project_id, &webhook_event).await {
                Some(cons)=> {
                    let _tes = self.kirim_notif(
                        &project_name, 
                        &webhook_event,
                        time,  
                        &user,  
                        cons).await;
                }
                None => return
            }    
        }
    }

    pub async fn kirim_notif(&self, project: &str, event: &str, created: &str, by: &str, connectors: Vec<Connector>) -> HandlerResult  {
        let time = format!("{}", DateTime::parse_from_rfc3339(created).unwrap()
            .with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap())
            .format("%d/%m/%Y %H:%M"));
        let mut evo = "".to_owned();

        match event{
            "jira:issue_created" => evo = "Issue Created".to_owned(),
            "jira:issue_updated" => evo = "Issue Updated".to_owned(),
            "jira:issue_deleted" => evo = "Issue Deleted".to_owned(),
            "comment_created" => evo = "Comment Created".to_owned(),
            "comment_updated" => evo = "Comment Updated".to_owned(),
            "comment_deleted" => evo = "Comment Deleted".to_owned(),
            _=> println!("no event")
        }
        
        for con in connectors {
            let text =  format!("New Jira Notification!\nProject: {}\nEvent: {}\nCreated at: {}\nBy: {}
            ", project, 
               evo,
               time,  
               by, 
            );
            if con.bot_type.to_lowercase().eq("telegram") {    
                let _send = teloxide::Bot::new(con.token).send_message(con.chatid, text).await;
            }  
            else if con.bot_type.to_lowercase().eq("slack") {
                let _res = Slack::new(con.token).unwrap()
                    .send(&PayloadBuilder::new()
                    .text(text)
                    .build()
                    .unwrap()).await;
            }    
        }
        Ok(())
    }

    pub async fn list_bucket_path(&self) -> Result<Vec<String>, ConnectorError>{    
        match self.s3.list_objects_v2(ListObjectsV2Request {
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
                    Ok(list)
                }
                
            },
            Err(e) => return Err(ConnectorError::RusError(e.to_string()))
        }
    }
    
    #[allow(warnings)]
    pub async fn get_yml(&self, paths: Vec<String>) ->  Result<Vec<Connector>, ConnectorError> {
        let mut cons: Vec<Connector> = Vec::new();
    
        for pa in paths {
            match self.s3.get_object(GetObjectRequest {
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

    #[allow(warnings)]
    pub async fn init_directory(&self) {
        match self.s3.head_object(HeadObjectRequest {
            bucket: BUCKET.to_string(),
            key: format!("{}/", FOLDER.to_owned()),
            ..Default::default()
        }).await {
            Ok(_) => { println!("Directory already exist") },
            Err(_) => {
                match self.s3.put_object(PutObjectRequest {
                    bucket: BUCKET.to_owned(),
                    key: format!("{}/", FOLDER.to_owned()),
                    body: None,
                    ..Default::default()
                }).await{
                    Ok(_) => println!("Directory successfully created"),
                    Err(e) => println!("{:?}", e.to_string())
                }
            } 
        }
    }
    
    pub async fn connector_exist(&self, name: &str) -> bool {
        match self.s3.head_object(HeadObjectRequest {
            bucket: BUCKET.to_string(),
            key: format!("{}/{}.yml", FOLDER.to_owned(), name.to_owned()),
            ..Default::default()
        }).await{
            Ok(_) => true,
            Err(_) => false
        }
    }
    
    pub async fn add_connector(&self, payload: &Connector) -> Result<String, ConnectorError> {
        if self.connector_exist(&payload.name).await {
            return Err(ConnectorError::ConCreateExist)
        } else {
            match self.s3.put_object(PutObjectRequest {
                bucket: BUCKET.to_owned(),
                key: format!("{}/{}.yml", FOLDER.to_owned(), payload.name),
                body: Some((serde_yaml::to_string(&payload).unwrap().into_bytes()).into()),
                ..Default::default()
            }).await{
                Ok(_) => return Ok("Connector successfuly created".to_owned()),
                Err(e) => return Err(ConnectorError::RusError(e.to_string()))
            }
        }
    }
    
    pub async fn delete_connector(&self, target_name: String) -> Result<String, ConnectorError> {
        if self.connector_exist(&target_name).await {
            match self.s3.delete_object( DeleteObjectRequest  {
                bucket: BUCKET.to_owned(),
                key: format!("{}/{}.yml", FOLDER.to_owned(), target_name),
                ..Default::default()
            }).await{
                Ok(_) => return Ok("Connector successfuly deleted".to_owned()),
                Err(e) => return Ok(e.to_string())
            }
        } else {
            return Err(ConnectorError::ConNotFound)
        }
    }
    
    pub async fn get_connectors(&self) -> Option<Vec<Connector>> {
        match self.list_bucket_path().await {
            Ok(path) => {
                match self.get_yml(path).await {
                    Ok(sip) => Some(sip),
                    Err(_e) => None
                }
            },
            Err(_e) =>  None
        }
    }
    
    #[allow(warnings)]
    pub async fn get_one_connector(&self, target_name: String) -> Option<Connector> {
        if self.connector_exist(&target_name).await {
            match self.s3.get_object(GetObjectRequest {
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
    
    pub async fn update_connector(&self, target_name: String, payload: &Connector) -> Result<String, ConnectorError> {
        if self.connector_exist(&target_name).await {
            if self.connector_exist(&payload.name).await && &payload.name != &target_name {
                return Err(ConnectorError::ConUpdateExist)
            } 
        
            if payload.bot_type.to_lowercase() == "telegram".to_owned() { 
                let bot = Bot::new(&payload.token.to_owned());

                if let Err(_e) = bot.get_me().await {
                    return Err(ConnectorError::TokenInval)
                }

                if let Err(_e) = bot.get_chat(payload.chatid.clone()).send().await {
                    return Err(ConnectorError::ChatidInval)
                }
            } else {
                match Slack::new(&payload.token) {
                    Ok(s)=> {
                        if let Err(_) = s.send(&PayloadBuilder::new()
                            .text("This is a connection test message")
                            .build()
                            .unwrap()).await {
                            return Err(ConnectorError::TokenInval)
                        }
                    },
                    Err(_) => return Err(ConnectorError::TokenInval)
                }
            };

            match self.s3.put_object(PutObjectRequest {
                bucket: BUCKET.to_string(),
                key: format!("{}/{}.yml", FOLDER.to_owned(), payload.name),
                body: Some((serde_yaml::to_string(&payload).unwrap().into_bytes()).into()),
                ..Default::default()
            }).await{
                Ok(_) => {
                    if &payload.name != &target_name {
                        match self.s3.delete_object(DeleteObjectRequest  {
                            bucket: BUCKET.to_owned(),
                            key: format!("{}/{}.yml", FOLDER.to_owned(), target_name),
                            ..Default::default()
                        }).await {
                            Ok(_) =>  return Ok("Connector successfuly updated! ps.new name".to_owned()),
                            Err(e) => return Ok(e.to_string())
                        }
                    } else {
                        return Ok("Connector successfuly updated!".to_owned())
                    }
                }, 
                Err(e) => return Ok(e.to_string())
            }
        } else {
            return Err(ConnectorError::ConNotFound)
        }
    }
    
    pub async fn find_connectors(&self, project_id: &str, event: &str) -> Option<Vec<Connector>> {
        match self.get_connectors().await {
            Some(cons) => {
                let filtered = cons
                .into_iter()
                .filter(|con| con.project.iter().any(|proyek|proyek.id == project_id)
                    && con.event.iter().any(|even| even.eq(&event))
                    && con.active
                )
                .collect::<Vec<_>>();
    
                if filtered.len() == 0 { None } 
                else{ return Some(filtered) }
            }, 
            None => None
        }
    }
}
