use chrono::{DateTime, FixedOffset};
use serde_json::Value;
use rusoto_s3::S3Client;
use teloxide::requests::Requester;

use crate::actions::{connector, log};
use crate::models::connector::Connector;

pub async fn process_event(db: &S3Client, val: Value) -> Result<String, String>{
  
    let webhook_event= val.get("webhookEvent").and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "".to_string());

    let project_id = val.get("issue").and_then(|issue|{
            issue.get("fields").and_then(|fields|{
                fields.get("project").and_then(|project|{
                    project.get("id").and_then(|v| v.as_str().map(String::from))
                })
            })    
        }).unwrap_or_else(|| "".to_string());

    let project_name = val.get("issue").and_then(|issue|{
            issue.get("fields").and_then(|fields|{
                fields.get("project").and_then(|project|{
                    project.get("name").and_then(|v| v.as_str().map(String::from))
                })
            })    
        }).unwrap_or_else(|| "".to_string());    

    let mut user = "u".to_string();

    if webhook_event.contains("issue") {
        user = val.get("user").and_then(|user|{
                    user.get("displayName").and_then(|v| v.as_str().map(String::from)) 
            }).unwrap_or_else(|| "".to_string());
    } else if webhook_event.contains("comment") {
        user = val.get("comment").and_then(|comment|{
                    comment.get("author").and_then(|author|{
                        author.get("displayName").and_then(|v| v.as_str().map(String::from)) 
                    })    
            }).unwrap_or_else(|| "".to_string());
    }

    match find_connectors(db, &project_id, &webhook_event).await {
        Some(cons)=> {
            let tes = kirim_notif(
                db,
                &project_name, 
                &webhook_event,
                "temporary",  
                &user,  
                cons).await;
            if tes.is_ok(){
                return Ok("Notif Send".to_string())
            } else {
                return Err("Send fail somehow".to_string())
            }

        }
        None => return Ok("No Connector Related Found".to_string())
    }    
   
}

pub async fn find_connectors(db: &S3Client, project_id: &str, event: &str) -> Option<Vec<Connector>> {
    match connector::get_connectors(&db).await {
        Ok(cons) => {
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
        Err(_)=> None
    }
}

pub async fn kirim_notif(db: &S3Client, project: &str, event: &str, created: &str, by: &str, connectors: Vec<Connector>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
    // let time = format!("{}", DateTime::parse_from_rfc3339(created).unwrap()
    //     .with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap())
    //     .format("%d/%m/%Y %H:%M"));
    let time = created.to_string();
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

    let twili = twilio::Client::new("AC43c81da3609460c9dad7db7e54866b57", "ecdc188c518d56fad25d7eafadcc10e8");
    let from = "whatsapp:+14155238886";
    
    
    for con in connectors {
        let mut attempt = 0;
        let text =  format!("New Jira Notification!\nProject: {}\nEvent: {}\nCreated at: {}\nBy: {}
        ", project, 
           evo,
           time,  
           by, 
        );
        if con.bot_type.to_lowercase().eq("telegram") {  
            loop {
                attempt+=1;
                let send = teloxide::Bot::new(con.token.clone()).send_message(con.chatid.clone(), text.clone()).await;
                if send.is_ok(){
                    log::write_log(&db, con.name, event.to_string(), "sent".to_string(), attempt, time.clone()).await;
                    break
                } else if attempt == 3{
                    log::write_log(&db, con.name, event.to_string(), "fail".to_string(), attempt, time.clone()).await;
                    break
                }
            }  
        }  
        else if con.bot_type.to_lowercase().eq("slack") {
            loop {
                attempt+=1;
                let send = slack_hook2::Slack::new(con.token.clone()).unwrap()
                    .send(&slack_hook2::PayloadBuilder::new()
                    .text(text.clone())
                    .build()
                    .unwrap()).await;
                if send.is_ok(){
                    log::write_log(&db, con.name, event.to_string(), "sent".to_string(), attempt, time.clone()).await;
                    break
                } else if attempt == 3{
                    log::write_log(&db, con.name, event.to_string(), "fail".to_string(), attempt, time.clone()).await;
                    break
                }
            }
        } else if con.bot_type.to_lowercase().eq("whatsapp") {   
            loop {
                attempt+=1;      
                let send = twili.send_message(twilio::OutboundMessage::new(from, &format!("whatsapp:{}", con.token), &text)).await;
                if send.is_ok(){
                    log::write_log(&db, con.name, event.to_string(), "sent".to_string(), attempt, time.clone()).await;
                    break
                } else if attempt == 3{
                    log::write_log(&db, con.name, event.to_string(), "fail".to_string(), attempt, time.clone()).await;
                    break
                }
            }
        }       
    }
    Ok(())
}
