use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connector{
    pub name: String,
    pub description: String,
    pub email: String,
    pub api_key: String,
    pub bot_type: String,
    pub active: String,
    pub telegram_chatid: String,
    pub project_id: Vec<String>,
    pub event: Vec<String>,
}

impl Connector{
    pub fn new(name:String, description: String, email: String, api_key: String, bot_type: String, telegram_chatid: String, project_id: Vec<String>, event:Vec<String>)-> Connector {
        Connector { 
            name, 
            description, 
            email, 
            api_key, 
            bot_type,
            active: "true".to_owned(), 
            telegram_chatid, 
            project_id, 
            event } 
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectorGet{
    #[serde(rename="_id")]
    pub id: mongodb::bson::oid::ObjectId,
    
    pub name: String,
    pub description: String,
    pub email: String,
    pub api_key: String,
    pub bot_type: String,
    pub active: String,
    pub telegram_chatid: String,
    pub project_id: Vec<String>,
    pub event: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectorUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub api_key: Option<String>,
    pub bot_type: Option<String>,
    pub active: Option<String>,
    pub telegram_chatid: Option<String>,
    pub project_id: Option<Vec<String>>,
    pub event: Option<Vec<String>>,
}

impl ConnectorUpdate{
    pub fn new(name:String, description: String, email: String, api_key: String, bot_type: String, telegram_chatid: String, project_id: Vec<String>, event:Vec<String>)-> Connector {
        Connector { 
            name, 
            description, 
            email, 
            api_key, 
            bot_type,
            active: "true".to_owned(), 
            telegram_chatid, 
            project_id, 
            event } 
    }
}