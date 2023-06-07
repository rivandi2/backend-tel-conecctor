use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct UserInput{
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct UserNew{
    pub username: String,
    pub password: String,
    pub created_at: DateTime<FixedOffset>,
    
    pub jira_email: Option<String>,
    pub jira_api_key: Option<String>,
    pub jira_url: Option<String>,
    pub webhook_url: Option<String>,
    pub webhook_functional: Option<bool>,
    pub webhook_last_check: Option<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct User{
    #[serde(rename="_id")]
    pub id: mongodb::bson::oid::ObjectId,

    pub username: String,
    pub password: String,
    pub created_at: DateTime<FixedOffset>,
    
    pub jira_email: Option<String>,
    pub jira_api_key: Option<String>,
    pub jira_url: Option<String>,
    pub webhook_url: Option<String>,
    pub webhook_functional: Option<bool>,
    pub webhook_last_check: Option<String>
}


