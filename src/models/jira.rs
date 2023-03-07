use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AvatarUrls{
    #[serde(rename="48x48")]
    big: String,
    
    #[serde(rename="24x24")]
    small: String,
    
    #[serde(rename="16x16")]
    smaller: String, 

    #[serde(rename="32x32")]
    medium: String,
} 

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectList {
   
    expand: String,

    #[serde(rename="self")]
    _self: String,

    #[serde(rename="id")]
    pub id: String,

    pub key: String,

    pub name: String,

    #[serde(rename="avatarUrls")]
    avatar_urls: AvatarUrls,

    #[serde(rename="projectTypeKey")]
    project_typekey: String, 

    simplified: bool, 

    style: String,

    #[serde(rename="isPrivate")]
    is_private: bool,

    #[serde(skip_serializing_if="Option::is_none")]
    properties: Option<Empty>,

    #[serde(rename="entityId", skip_serializing_if="Option::is_none")]
    entity_id: Option<String>,

    #[serde(skip_serializing_if="Option::is_none")]
    uuid: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventType{
    pub id: i32,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaringProject{
    pub id: String,
    pub name: String,
}

