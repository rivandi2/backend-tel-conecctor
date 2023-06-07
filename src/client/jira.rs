use std::str::FromStr;

use actix_web::web;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::UpdateOptions;
use serde_json::Value;

use crate::models::jira::{SaringProject, ProjectList, Webhook, Filter};
use crate::errortype::JiraError;
use crate::models::user::User;
use crate::routes::jira::WebhookInput;

#[derive(Clone)]
pub struct Client {
    pub reqwest: reqwest::Client
}

impl Client{
    pub fn new() -> Self {
     return Self { 
            reqwest: reqwest::Client::new()
        }
    }

    pub async fn get_projects(&self, user: User) -> Result<Vec<SaringProject>, JiraError>{
        match self.get_request(
            user.jira_email.unwrap(),
            user.jira_api_key.unwrap(),
            format!("https://{}/rest/api/3/project", user.jira_url.unwrap()))
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

    pub async fn create_webhook(&self, mongodb: &mongodb::Client,  webhook: &WebhookInput, user: User) -> Result<String, JiraError>{
        let source_url = format!("https://{}/rest/webhooks/1.0/webhook", webhook.jira_url);
        let payload = Webhook {
            name: format!("User {}'s webhook", user.id.to_hex()).to_string(),
            url: format!("https://atlassian-connector-api.dev-domain.site/{}", user.id.to_hex()).to_string(),
            events: vec!["jira:issue_created".to_string(),"jira:issue_updated".to_string(),"jira:issue_deleted".to_string(),
                    "comment_created".to_string(),"comment_updated".to_string(),"comment_deleted".to_string()],
            filters: Filter{
                issue_related_events_section: "".to_string()
            },
            exclude_body: false,
            enabled: true
        };
        
        match self.reqwest
            .post(source_url)
            .basic_auth(webhook.email.to_string(), Some(webhook.api_key.to_string()))
            .json(&payload)
            .send()
            .await {
            Ok(text) => {
                    let message=  text.text().await.unwrap();

                    if message.contains("Webhook with the same URL, set of events, filters"){
                        return Err(JiraError::ErrorMessage("You have already created a webhook for this account".to_string()))
                    } else if message.contains("Page unavailable") {
                        return Err(JiraError::ErrorMessage("Jira url not found, make sure to enter the correct path".to_string()))
                    } else if message.contains("Client must be authenticated to access this resource"){
                        return Err(JiraError::ErrorMessage("Incorrect jira email or api key".to_string()))
                    } else {
                        let val: Value = serde_json::from_str(&message).unwrap();

                        let webhook_url = val.get("self").and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| "".to_string());

                        match mongodb
                            .database("telconnect")
                            .collection::<User>("users")
                            .update_one( doc! { "_id": ObjectId::from_str(&user.id.to_hex()).unwrap() }, 
                                doc!{
                                    "$set": {
                                        "jira_email": &webhook.email,
                                        "jira_api_key": &webhook.api_key,
                                        "jira_url": &webhook.jira_url,
                                        "webhook_url": webhook_url
                                    }
                                }, 
                        UpdateOptions::builder()
                                    .upsert(false) 
                                    .build()).await{
                                Ok(o) => {
                                    println!("{:?}", o);
                                    return Ok("Webhook successfully created".to_string())
                                },
                                Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
                        };
                    }
            }
            Err (_e) => return Err(JiraError::ErrorMessage("Error sending request to the url, make sure the url is from an atlassian domain".to_string()))
        }
    }

    pub async fn delete_webhook(&self, mongodb: &mongodb::Client, user: User) -> Result<String, JiraError>{
        match self.reqwest
            .delete(user.webhook_url.unwrap())
            .basic_auth(user.jira_email.unwrap().to_string(), Some(user.jira_api_key.unwrap().to_string()))
            .send()
            .await {
        Ok(text) => {
                match mongodb
                    .database("telconnect")
                    .collection::<User>("users")
                    .update_one( doc! { "_id": ObjectId::from_str(&user.id.to_hex()).unwrap() }, 
                        doc!{
                            "$set": {
                                "jira_email": None::<String>,
                                "jira_api_key": None::<String>,
                                "jira_url": None::<String>,
                                "webhook_url": None::<String>,
                                "webhook_functional": None::<bool>,
                                "webhook_last_check": None::<String>
                            }
                        }, 
                UpdateOptions::builder()
                            .upsert(false) 
                            .build()).await{
                        Ok(_) => {
                            if text.status() == 404 {
                                return Ok("Your webhook has been deleted by an unknown party".to_string())
                            } else {
                                return Ok("Webhook successfully deleted".to_string())
                            }
                        },
                        Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
                };
                
        }
        Err (_e) => return Err(JiraError::ErrorMessage("Error sending request to the url, make sure the url is from an atlassian domain".to_string()))
        }
        
    }

    pub async fn check_webhook(&self, mongodb: &mongodb::Client, user: User) -> Result<String, JiraError>{
        match self.reqwest
            .get(user.webhook_url.unwrap())
            .basic_auth(user.jira_email.unwrap().to_string(), Some(user.jira_api_key.unwrap().to_string()))
            .send()
            .await {
        Ok(text) => {
            if text.status() == 404 {
                match mongodb
                    .database("telconnect")
                    .collection::<User>("users")
                    .update_one( doc! { "_id": ObjectId::from_str(&user.id.to_hex()).unwrap() }, 
                        doc!{
                            "$set": {
                                "jira_email": None::<String>,
                                "jira_api_key": None::<String>,
                                "jira_url": None::<String>,
                                "webhook_url": None::<String>,
                                "webhook_functional": None::<bool>,
                                "webhook_last_check": None::<String>
                            }
                        }, 
                UpdateOptions::builder()
                            .upsert(false) 
                            .build()).await{
                    Ok(_) => {
                        return Err(JiraError::ErrorMessage("Your webhook has been deleted by an unknown party, please create a new webhook".to_string()))
                    },
                    Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
                } 
            } 

            let message=  text.text().await.unwrap();
            let check: Webhook = serde_json::from_str(&message).unwrap();

            let mut err= String::new();

            let url = format!("https://atlassian-connector-api.dev-domain.site/{}", user.id.to_hex());
            
            let vector2 = vec![
                String::from("jira:issue_created"),
                String::from("jira:issue_updated"),
                String::from("jira:issue_deleted"),
                String::from("comment_created"),
                String::from("comment_updated"),
                String::from("comment_deleted"),
            ];

            let set1: std::collections::HashSet<_> = check.events.iter().cloned().collect();
            let set2: std::collections::HashSet<_> = vector2.into_iter().collect();

            if set1 != set2 {
                err.push_str("Webhook event is not correct,");
            } 
            if url != check.url {
                err.push_str("Webhook endpoint url is not correct,");
            } 
            if check.enabled == false{
                err.push_str("Webhook is not enabled,");
            }
            if check.exclude_body == true{
                err.push_str("Exclude body must be unchecked,");
            }

            let mut status = true;
            let now  = chrono::Utc::now()
                .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).unwrap())
                .format("%d %b %Y, %H:%M:%S").to_string();

            if err.is_empty(){ status = true;
            } else { status = false }   

            match mongodb
                    .database("telconnect")
                    .collection::<User>("users")
                    .update_one( doc! { "_id": ObjectId::from_str(&user.id.to_hex()).unwrap() }, 
                        doc!{
                            "$set": {
                                "webhook_functional": status,
                                "webhook_last_check": now
                            }
                        }, 
                UpdateOptions::builder()
                            .upsert(false) 
                            .build()).await{
                    Ok(_) => {
                        if status {
                            return Ok("Webhook status functional".to_string());  
                        } else {
                            return Ok(err.to_string())
                        }
                    },
                    Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
                } 
        }
        Err (_e) => return Err(JiraError::ErrorMessage("Error sending request to the url, make sure the url is from an atlassian domain".to_string()))
        }
            
    }

    pub async fn repair_webhook(&self, mongodb: &mongodb::Client, user: User) -> Result<String, JiraError>{
        let fix = Webhook {
            name: format!("User {}'s webhook", user.id.to_hex()).to_string(),
            url: format!("https://atlassian-connector-api.dev-domain.site/{}", user.id.to_hex()).to_string(),
            events: vec!["jira:issue_created".to_string(),"jira:issue_updated".to_string(),"jira:issue_deleted".to_string(),
                    "comment_created".to_string(),"comment_updated".to_string(),"comment_deleted".to_string()],
            filters: Filter{
                issue_related_events_section: "".to_string()
            },
            exclude_body: false,
            enabled: true
        };

        match self.reqwest
            .put(user.webhook_url.unwrap())
            .basic_auth(user.jira_email.unwrap().to_string(), Some(user.jira_api_key.unwrap().to_string()))
            .json(&fix)
            .send()
            .await {
                Ok(res) => {
                    if res.status() == 404 {
                        match mongodb
                            .database("telconnect")
                            .collection::<User>("users")
                            .update_one( doc! { "_id": ObjectId::from_str(&user.id.to_hex()).unwrap() }, 
                                doc!{
                                    "$set": {
                                        "jira_email": None::<String>,
                                        "jira_api_key": None::<String>,
                                        "jira_url": None::<String>,
                                        "webhook_url": None::<String>,
                                        "webhook_functional": None::<bool>,
                                        "webhook_last_check": None::<String>
                                    }
                                }, 
                        UpdateOptions::builder()
                                    .upsert(false) 
                                    .build()).await{
                            Ok(_) => {
                                return Err(JiraError::ErrorMessage("Your webhook has been deleted by an unknown party, please create a new webhook".to_string()))
                            },
                            Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
                        } 
                    } 
                
                    return Ok("Webhook successfully modified".to_string())
                }, 
                Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))

        }
        
    }
    
}