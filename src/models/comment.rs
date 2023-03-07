use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project{
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fields{
    pub project: Project,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue{
    pub fields: Fields,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author{
    #[serde(rename="displayName")]
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment{
    pub author: Author,
    pub created: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body{
    #[serde(rename="webhookEvent")]
    pub webhook_event: String,

    pub comment: Comment,

    pub issue: Issue,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Data{
    pub body: Body,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcaraComment{
    pub data: Data,
}