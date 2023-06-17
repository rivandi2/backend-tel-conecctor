use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ConnectorInput{
    pub name: String,
    pub description: String,
    pub token: String,
    pub chatid: String,
    pub active: bool,
    pub schedule: bool,
    pub duration: String,
    pub project: Vec<Project>,
    pub event: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Connector{
    pub name: String,
    pub description: String,
    pub token: String,
    pub chatid: String,
    pub active: bool,
    pub schedule: bool,
    pub duration: String,
    pub project: Vec<Project>,
    pub event: Vec<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: Option<DateTime<FixedOffset>>
}

#[derive(Debug, Serialize, Deserialize,  PartialEq, Eq, Clone)]
pub struct Project{
    pub id: String,
    pub name: String
}