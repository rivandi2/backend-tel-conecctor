use chrono::TimeZone;
use serde_json::Value;
use rusoto_s3::S3Client;
use teloxide::requests::Requester;

use crate::actions::{connector, log};
use crate::models::{connector::Connector, event::Event};

pub async fn process_event(db: &S3Client, val: Value, id: String) -> Result<String, String>{
    let mut event = Event {
        timestamp: val.get("timestamp").and_then(|v| v.as_i64())
                    .unwrap_or(0) as i64,

        webhook_event: val.get("webhookEvent").and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "".to_string()),

        project_id: val.get("issue").and_then(|issue|{
                issue.get("fields").and_then(|fields|{
                    fields.get("project").and_then(|project|{
                        project.get("id").and_then(|v| v.as_str().map(String::from))
                    })
                })    
            }).unwrap_or_else(|| "".to_string()),

        project_name: val.get("issue").and_then(|issue|{
                issue.get("fields").and_then(|fields|{
                    fields.get("project").and_then(|project|{
                        project.get("name").and_then(|v| v.as_str().map(String::from))
                    })
                })    
            }).unwrap_or_else(|| "".to_string()),

        issue_key: val.get("issue").and_then(|issue|{
                issue.get("key").and_then(|v| v.as_str().map(String::from))       
            }).unwrap_or_else(|| "".to_string()),   

        summary: val.get("issue").and_then(|issue|{
            issue.get("fields").and_then(|fields|{
                    fields.get("summary").and_then(|v| v.as_str().map(String::from))
                })    
            }).unwrap_or_else(|| "".to_string()),   

        issue_type: val.get("issue").and_then(|issue|{
                issue.get("fields").and_then(|fields|{
                    fields.get("issuetype").and_then(|issuetype|{
                        issuetype.get("name").and_then(|v| v.as_str().map(String::from))
                    })
                })    
            }).unwrap_or_else(|| "".to_string()), 

        assignee: val.get("issue").and_then(|issue|{
                issue.get("fields").and_then(|fields|{
                    fields.get("assignee").and_then(|assignee|{
                        assignee.get("displayName").and_then(|v| v.as_str().map(String::from))
                    })
                })    
            }).unwrap_or_else(|| "-".to_string()), 

        user: String::new(),

        changes: String::new(),

        comment:  String::new(),                    
    };
        
    if event.webhook_event.contains("issue") {
        event.user = val.get("user").and_then(|user|{
                    user.get("displayName").and_then(|v| v.as_str().map(String::from)) 
            }).unwrap_or_else(|| "".to_string());

        if event.webhook_event.contains("updated") {
            let items = val.get("changelog").and_then(|changelog| { 
                    changelog.get("items").and_then(|items| items.as_array())
                }).unwrap();

            for it in items{
                event.changes.push_str(&format!("{}: {} -> {}\n", 
                    it.get("field").and_then(|v| v.as_str().map(String::from)).unwrap_or("".to_string()), 
                    it.get("fromString").and_then(|v| v.as_str().map(String::from)).unwrap_or("-".to_string()), 
                    it.get("toString").and_then(|v| v.as_str().map(String::from)).unwrap_or("-".to_string()))
                );
            }    
        }
    } else if event.webhook_event.contains("comment") {
        event.user = val.get("comment").and_then(|comment|{
                comment.get("updateAuthor").and_then(|update_author|{
                    update_author.get("displayName").and_then(|v| v.as_str().map(String::from)) 
                })    
            }).unwrap_or_else(|| "".to_string());

        event.comment = val.get("comment").and_then(|comment|{
                comment.get("body").and_then(|v| v.as_str().map(String::from)) 
            }).unwrap_or_else(|| "".to_string());
    }

    match find_connectors(db, &event.project_id, &event.webhook_event, id.clone()).await {
        Some(cons)=> {
            send_notification(db, event,cons,id).await;
            return Ok("Event processed".to_string())
        }
        None => return Ok("No Connector Related Found".to_string())
    }    
   
}

pub async fn find_connectors(db: &S3Client, project_id: &str, event: &str, id: String) -> Option<Vec<Connector>> {
    match connector::get_connectors(&db, id).await {
        Ok(cons) => {
            let now = (chrono::Utc::now() + chrono::Duration::hours(7)).naive_utc().time();

            let filtered = cons
                .into_iter()
                .filter(|con| con.project.iter().any(|proyek|proyek.id == project_id)
                    && con.event.iter().any(|even| even.eq(&event))
                    && con.active
                    && if con.schedule {
                        let split: Vec<&str> = con.duration.split('-').collect();
                        let s0 = chrono::NaiveTime::parse_from_str(split[0], "%H:%M").unwrap();
                        let s1 = chrono::NaiveTime::parse_from_str(split[1], "%H:%M").unwrap();

                        if s0 > s1 {
                            if (now >= s0) || (s0 >= now && now <= s1) {
                                return true
                            }else {
                                return false
                            }
                        } else {
                            if now >= s0 && now <= s1 {   
                                return true
                            } else {
                                return false
                            }
                        }
                    } else {
                        return true
                    }
                )
                .collect::<Vec<_>>();

            if filtered.len() == 0 { None } 
            else{ return Some(filtered) }
        }, 
        Err(_)=> None
    }
}

pub async fn send_notification(db: &S3Client, event: Event, connectors: Vec<Connector>, id: String ) {
    let time = chrono::Utc.timestamp_millis_opt(event.timestamp).unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).unwrap())
        .format("%d/%m/%Y %H:%M").to_string();
    let mut evo = "".to_owned();

    match event.webhook_event.as_ref() {
        "jira:issue_created" => evo = "created new issue".to_owned(),
        "jira:issue_updated" => evo = "made changes on an issue".to_owned(),
        "jira:issue_deleted" => evo = "deleted an issue".to_owned(),
        "comment_created" => evo = "created new comment".to_owned(),
        "comment_updated" => evo = "updated a comment".to_owned(),
        "comment_deleted" => evo = "deleted a comment".to_owned(),
        _=> println!("no event")
    }

    let mut text = "".to_string();
    
    if event.webhook_event.eq("jira:issue_created") || event.webhook_event.eq("jira:issue_deleted") {
        text = format!("{} {} in project {}\n\nIssue: {} {}\nIssue type: {}\nAssignee: {}",
            event.user,
            evo,
            event.project_name,
            event.issue_key,
            event.summary,
            event.issue_type,
            event.assignee
        );
    } else if event.webhook_event.eq("jira:issue_updated"){
        text = format!("{} {} in project {}\n\nAffected issue: {} {}\nCHANGELOG\n{}",
           event.user,
           evo,
           event.project_name, 
           event.issue_key,
           event.summary,
           event.changes
        );
    } else {
        text = format!("{} {} on an issue in project {}\n\nIssue: {} {}\nComment: {:?}\n",
            event.user,
            evo,
            event.project_name,
            event.issue_key,
            event.summary,
            event.comment
        );
    }
   
    for con in connectors {
        let mut attempt = 0;
        loop {
            attempt+=1;
            let send = teloxide::Bot::new(con.token.clone()).send_message(con.chatid.clone(), &text.clone()).await;
            if send.is_ok(){
                log::write_log(&db, con.name, event.webhook_event.to_string(), "sent".to_string(), attempt, time.clone(), &id).await;
                break
            } else if attempt == 3{
                log::write_log(&db, con.name, event.webhook_event.to_string(), "fail".to_string(), attempt, time.clone(), &id).await;
                break
            }
        }  
    }
}