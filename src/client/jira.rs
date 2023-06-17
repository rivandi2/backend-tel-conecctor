use std::str::FromStr;

use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::UpdateOptions;
use serde_json::Value;

use crate::models::jira::{Project, ProjectList, Webhook, Filter};
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

    pub async fn get_projects(&self, user: User) -> Result<Vec<Project>, JiraError>{
        match self.reqwest
            .get(format!("https://{}/rest/api/3/project", user.jira_url.unwrap()))
            .basic_auth(user.jira_email.unwrap(), Some(user.jira_api_key.unwrap()))
            .send()
            .await {
            Ok(sip) => {
                let text = sip.text().await.unwrap();
                if text.contains("Basic authentication with passwords is deprecated") {
                    return Err(JiraError::ApiKeyError)
                } 
                let list: Vec<ProjectList> = serde_json::from_str(&text).unwrap();
                if list.len() == 0 {return Err(JiraError::EmailError)};

                let saring: Vec<Project> = list.into_iter()
                    .map(|po| Project { id: po.id, name: po.name })
                    .collect();
                return Ok(saring)
            }
            Err (e) => return Err(JiraError::ErrorMessage(e.to_string()))
        }
    }
    
    pub async fn create_webhook(&self, mongodb: &mongodb::Client,  webhook: &WebhookInput, user: User) -> Result<String, JiraError>{
        let source_url = format!("https://{}/rest/webhooks/1.0/webhook", webhook.jira_url);
        let payload = Webhook {
            name: format!("User {}'s webhook", user.id.to_hex()).to_string(),
            url: format!("https://atlassian-connector-api.dev-domain.site/event/{}", user.id.to_hex()).to_string(),
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
                            } else if text.status() == 401 {
                                return Ok("Unable to confirm deletion of webhook as your jira credentials may have been changed. Please confirm webhook deletion by yourself".to_string())
                            }  else {
                                return Ok("Webhook successfully deleted".to_string())
                            }
                        },
                        Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
                };
                
        }
        Err (_e) => {
            let _res = mongodb
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
                        .build()).await;
            return Err(JiraError::ErrorMessage("Error sending request to the url, make sure the url is from an atlassian domain".to_string()))
            }
        }
        
    }

    pub async fn check_webhook(&self, mongodb: &mongodb::Client, user: User) -> Result<String, JiraError>{
        match self.reqwest
            .get(user.webhook_url.clone().unwrap())
            .basic_auth(user.jira_email.clone().unwrap().to_string(), Some(user.jira_api_key.clone().unwrap().to_string()))
            .send()
            .await {
        Ok(res) => {
           match self.check_response(res, mongodb, user).await{
                Ok(o) => return Ok(o),
                Err(e) => return Err(e)
           }
        }
        Err (_e) => return Err(JiraError::ErrorMessage("Error sending request to the url, make sure the url is from an atlassian domain".to_string()))
        }
    }

    pub async fn repair_webhook(&self, mongodb: &mongodb::Client, user: User) -> Result<String, JiraError>{
        let fix = Webhook {
            name: format!("User {}'s webhook", user.id.to_hex()).to_string(),
            url: format!("https://atlassian-connector-api.dev-domain.site/event/{}", user.id.to_hex()).to_string(),
            events: vec!["jira:issue_created".to_string(),"jira:issue_updated".to_string(),"jira:issue_deleted".to_string(),
                    "comment_created".to_string(),"comment_updated".to_string(),"comment_deleted".to_string()],
            filters: Filter{
                issue_related_events_section: "".to_string()
            },
            exclude_body: false,
            enabled: true
        };

        match self.reqwest
            .put(user.webhook_url.clone().unwrap())
            .basic_auth(user.jira_email.clone().unwrap().to_string(), Some(user.jira_api_key.clone().unwrap().to_string()))
            .json(&fix)
            .send()
            .await {
                Ok(res) => {
                    match self.check_response(res, mongodb, user).await {
                        Ok(o) => {
                            if o.contains("Webhook status functional"){
                                return Ok("Webhook has been successfully modified, webhook status is now functional".to_string())
                            } else {
                                return Ok(o)
                            }
                        },
                        Err(e) => return Err(e)
                    }              
                }, 
                Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
        }
        
    }

    pub async fn check_response(&self, res: reqwest::Response, mongodb: &mongodb::Client, user: User) -> Result<String, JiraError> {
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
                    return Err(JiraError::ErrorMessage("Your webhook has been deleted by an unknown party".to_string()))
                },
                Err(e) => return Err(JiraError::ErrorMessage(e.to_string()))
            } 
        } else {
            let message= res.text().await.unwrap();
            let check: Webhook = serde_json::from_str(&message).unwrap();

            let mut err= String::new();

            let url = format!("https://atlassian-connector-api.dev-domain.site/event/{}", user.id.to_hex());
            
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

            if err.is_empty(){ status = true } 
            else { status = false }   

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
    }
    
}