use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project{
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fields{
    pub project: Project,
    pub created: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue{
    pub fields: Fields,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User{
    #[serde(rename="displayName")]
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body{
    #[serde(rename="webhookEvent")]
    pub webhook_event: String,

    pub user: User,

    pub issue: Issue,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Data{
    pub body: Body,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Acara{
    pub data: Data,
}


