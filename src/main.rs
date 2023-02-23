use crate::serde_json::Value;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

use std::env;
use serde::{Serialize, Deserialize};
use serde_json;
use thiserror::Error;
use dotenv::dotenv;
use mongodb::bson::{Document, doc};
use mongodb::options::FindOptions;
use futures::TryStreamExt;
use teloxide::prelude::*;

mod models;
use models::jira::{ProjectList, EventType};
use models::event::HookdeckEvents;
use models::issue::{Acara, Channels};
use models::comment::AcaraComment;

#[derive(Debug, Serialize, Deserialize)]
pub struct LastEvent{
    pub id: String
}

#[derive(Error, Debug)]
pub enum JiraError {
    #[error("Placehold error")] ProjectFound(#[from] reqwest::Error),
    #[error("Request dapat ditemukan")] RequestFail,
    #[error("Tidak dapat diubah ke JSON text")] TextChange,
    #[error("Tidak bisa masukin ke vector")] VectorFail(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Klien 
{
    pub reqwest: reqwest::Client,
    pub mongodb: mongodb::Client,
    pub bot: teloxide::Bot

}

impl Klien
{
    pub fn new() -> Self
    {
        return Self{
            reqwest: reqwest::Client::new(),
            mongodb: futures::executor::block_on(mongodb::Client::with_uri_str("mongodb+srv://rivandi:Z0BDxmHQc9k237ES@cluster0.kixsqjq.mongodb.net/?retryWrites=true&w=majority")).unwrap(),
            bot: Bot::from_env()
        }
    }

    pub async fn get_projects(&self) -> Result<Vec<ProjectList>, JiraError>{
        let result = self.get_request(
            env::var("USER_NAME").expect("USER_NAME not found").to_owned(), 
            env::var("PASSWORD").expect("PASSWORD not found").to_owned(), 
            "https://telkomdevelopernetwork.atlassian.net/rest/api/3/project".to_owned())
            .await;
        match result {
            Ok(text) => {
                    let list: Vec<ProjectList> = serde_json::from_str(&text).unwrap();
                    return Ok(list)
                    }
            Err (e) => return Err(e)
        }
    }

    pub async fn get_events_type(&self) -> Result<Vec<EventType>, JiraError>{
        let result = self.get_request(
            env::var("USER_NAME").expect("USER_NAME not found").to_owned(), 
            env::var("PASSWORD").expect("PASSWORD not found").to_owned(), 
            "https://telkomdevelopernetwork.atlassian.net/rest/api/3/events".to_owned())
            .await;
        match result {
            Ok(text) => {
                    let list: Vec<EventType> = serde_json::from_str(&text).unwrap();
                    return Ok(list)
                    }
            Err (e) => return Err(e)
        }
    }

    pub async fn get_hookdeck_events(&self) -> Result<HookdeckEvents, JiraError> {
        let result = self.get_request(
            env::var("HOOKDECK_API_KEY").expect("APIKEYNOTFOUND").to_owned(), 
            "".to_owned(), 
            "https://api.hookdeck.com/2023-01-01/events".to_owned())
            .await;
        match result {
            Ok(text) => {
                    let list: HookdeckEvents = serde_json::from_str(&text).unwrap();
                    return Ok(list)
                    }
            Err (e) => return Err(e)
        }
    }

    pub async fn get_hookdeck_events_data(&self, id: &str) -> Result<Value, JiraError> {
        let url = format!("https://api.hookdeck.com/2023-01-01/events/{}", id);
        let result = self.get_request(
            env::var("HOOKDECK_API_KEY").expect("APIKEYNOTFOUND").to_owned(), 
            "".to_owned(), 
            url)
            .await;
        match result {
            Ok(text) => {
                    let list: Value = serde_json::from_str(&text).unwrap();
                    return Ok(list)
                    }
            Err (e) => return Err(e)
        }

    }

    pub async fn get_request(&self, username: String, password: String, url: String) -> Result<String, JiraError>
    {
        let response = self.reqwest
            .get(url)
            .basic_auth(username, Some(password))
            .send()
            .await;
        if response.is_err() {
            return Err(JiraError::RequestFail);
        } else 
            {
                let text = response.unwrap()
                    .text()
                    .await;
                match text {
                    Ok(sip) => return Ok(sip),
                    Err(_e) => return Err(JiraError::TextChange)
                }
            }
    }

    pub async fn add_projects(&self, pro: Vec<ProjectList>) {
        let coll = self.mongodb.database("telcon").collection::<Document>("projects");
        let _drp = coll.delete_many(doc!{}, None).await;
        for pr in pro {
            let docu = doc! {
                "id": pr.id,
                "key": pr.key,
                "name": pr.name
            };
            let _result = coll.insert_one(docu, None).await;      
        }
    }

    pub async fn add_event(&self, val: Value) {
        let coll = self.mongodb.database("telcon").collection::<Value>("event");
        let _drp = coll.delete_many(doc!{}, None).await;
        let _res = coll.insert_one(val, None).await;
    }

    pub async fn filter_events(&self) {
        let col1 = self.mongodb.database("telcon").collection::<Document>("event");
        let filter1 = doc! {
            "data.headers.user-agent": "Atlassian Webhook HTTP Client",
        };
        let options = FindOptions::builder().projection(doc! {
            "data.body.webhookEvent": 1,
            "_id": 0
        }).build();
        let cursor = col1.find(filter1, options).await;
        let doc = cursor.unwrap().try_collect().await.unwrap_or_else(|_| vec![]);

        if doc.is_empty(){
            let _drp = col1.delete_many(doc!{}, None).await;
        } else {
            let text = serde_json::to_string(&doc);
            let isi = text.unwrap().to_lowercase();
            
            if isi.contains("issue") {
                let issueoptions = FindOptions::builder().projection(doc! {
                    "data.body.issue.fields.project.id":1,
                    "data.body.issue.fields.project.name":1,
                    "data.body.webhookEvent": 1,
                    "data.body.issue.fields.created": 1,
                    "data.body.user.displayName": 1,
                    "_id": 0        
                }).build();
                let cursorisu = col1.find(None, issueoptions).await;
                let docisu = cursorisu.unwrap().try_collect().await.unwrap_or_else(|_| vec![]);
                let textisu = serde_json::to_string(&docisu);
                let tex: Vec<Acara> = serde_json::from_str(&textisu.unwrap()).unwrap();

                let tex2 = self.find_channels(&tex[0].data.body.issue.fields.project.id, &tex[0].data.body.webhook_event).await;
                let _tes = self.kirim_notif(
                    &tex[0].data.body.issue.fields.project.name, 
                    &tex[0].data.body.webhook_event,
                    &tex[0].data.body.issue.fields.created,  
                    &tex[0].data.body.user.display_name,  
                    tex2).await;
            }
            else if isi.contains("comment") {
                let commentoptions = FindOptions::builder().projection(doc! {
                    "data.body.issue.fields.project.id":1,
                    "data.body.issue.fields.project.name":1,
                    "data.body.webhookEvent": 1,
                    "data.body.comment.created": 1,
                    "data.body.comment.author.displayName": 1,
                    "_id": 0        
                }).build();
                let cursorcomment = col1.find(None, commentoptions).await;
                let doccomment = cursorcomment.unwrap().try_collect().await.unwrap_or_else(|_| vec![]);
                let textcomment = serde_json::to_string(&doccomment);
                let tex: Vec<AcaraComment> = serde_json::from_str(&textcomment.unwrap()).unwrap();

                let tex2 = self.find_channels(&tex[0].data.body.issue.fields.project.id, &tex[0].data.body.webhook_event).await;
                let _tes = self.kirim_notif(
                    &tex[0].data.body.issue.fields.project.name, 
                    &tex[0].data.body.webhook_event,
                    &tex[0].data.body.comment.created,  
                    &tex[0].data.body.comment.author.display_name,
                    tex2).await;
            }
                let _drp = col1.delete_many(doc!{}, None).await;
        }
        
    }

    pub async fn find_channels(&self, project: &str, event: &str) -> Vec<Channels>{
        let col2 = self.mongodb.database("telcon").collection::<Document>("channels");
        let filter = doc! {
            "project_id": project,
            "event": event,
            "active": "true"
        };
        let options2 = FindOptions::builder().projection(doc! {
            "_id": 0
        }).build();
        let cursor2 = col2.find(filter, options2).await;
        let doc2 = cursor2.unwrap().try_collect().await.unwrap_or_else(|_| vec![]);
        let text2 = serde_json::to_string(&doc2);
        let tex2: Vec<Channels> = serde_json::from_str(&text2.unwrap()).unwrap();

        return tex2
    }


    pub async fn kirim_notif(&self, project: &str, event: &str, created: &str, by: &str, channels: Vec<Channels>) -> HandlerResult  {
        for cha in channels {
            let text =  format!("New Jira Notification!\nProject: {:?}\nEvent: {:?}\nCreated at: {:?}\nBy: {:?}
                                        ", project, 
                                           event,
                                           created,  
                                           by, 
                                        );
            let _send = self.bot.send_message(cha.telegram_chatid, text).await?;                        
        }
        Ok(())
    }

    pub async fn check_new_event (&self, eid: &str) -> String {
        let col1 = self.mongodb.database("telcon").collection::<Document>("lastevent");
        let options = FindOptions::builder().projection(doc! {
            "_id": 0
        }).build();
        let cursor = col1.find(None, options).await;
        let doc = cursor.unwrap().try_collect().await.unwrap_or_else(|_| vec![]);

        if doc.is_empty(){
            let _tes = col1.insert_one(doc!{"id": eid}, None).await;
            return "baru".to_owned()
        } else {
            let text = serde_json::to_string(&doc);
            let tex: Vec<LastEvent>  = serde_json::from_str(&text.unwrap()).unwrap();

            if tex[0].id.eq(&eid) {
                return "lama".to_owned()
            } else {
                let batas = &tex[0].id;
                let _drp = col1.delete_many(doc!{}, None).await;
                let _tes = col1.insert_one(doc!{"id": eid}, None).await;

                return batas.to_owned()
            }
            
        }

    }

}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let klien = Klien::new();
    // let project = klien.get_projects().await;
    // let eventlist = klien.get_events_type().await;
    // let _tes = klien.add_projects(project.unwrap()).await;

    //tes ini
    run_new(klien).await;
}

pub async fn run_new(klien: Klien) {
    let events = klien.get_hookdeck_events().await;
    let events = events.unwrap();

    let batas = klien.check_new_event(&events.models[0].id).await;
    match batas.as_ref() {
        "lama" => println!("Tidak ada event baru"),
        _ => { 
            let mut i = 0;
                while &events.models[i].id != &batas && i < 10{
                    let data= klien.get_hookdeck_events_data(&events.models[i].id).await;
                        match data {
                            Ok(val) => { klien.add_event(val).await;
                                                klien.filter_events().await; 
                            },
                            Err(_e) => println!("Error"),
                        };
                    i+=1;
            }
            println!("{} event baru", i);
        }
    }
}