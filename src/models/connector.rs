use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Connector{
    pub name: String,
    pub description: String,
    pub email: String,
    pub api_key: String,
    pub bot_type: String,
    pub token: String,
    pub chatid: String,
    pub active: bool,
    pub schedule: bool,
    pub duration: String,
    pub project: Vec<ProjectID>,
    pub event: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize,  PartialEq, Eq, Clone)]
pub struct ProjectID {
    pub id: String,
    pub name: String
}