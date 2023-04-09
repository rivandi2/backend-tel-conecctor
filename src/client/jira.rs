use crate::models::jira::{SaringProject, ProjectList};
use crate::errortype::JiraError;

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

    pub async fn get_projects(&self, email: &str, key: &str) -> Result<Vec<SaringProject>, JiraError>{
        match self.get_request(
            email.to_owned(),
            key.to_owned(),
            "https://telkomdevelopernetwork.atlassian.net/rest/api/3/project".to_owned())
            .await {
            Ok(text) => {
                    if text.contains("Basic authentication with passwords is deprecated") {
                        return Err(JiraError::ApiKeyError)
                    } 
                    let list: Vec<ProjectList> = serde_json::from_str(&text).unwrap();
                    if list.len() == 0 {return Err(JiraError::EmailError)};
    
                    let saring: Vec<SaringProject> = list.into_iter()
                        .map(|po| SaringProject { id: po.id, name: po.name })
                        .collect();
                    return Ok(saring)
            }
            Err (e) => return Err(JiraError::ErrorMessage(e.to_string()))
        }
    }
    
    pub async fn get_request(&self, username: String, password: String, url: String) -> Result<String, JiraError> {
        let response = self.reqwest
            .get(url)
            .basic_auth(username, Some(password))
            .send()
            .await;
        if response.is_err() {
            return Err(JiraError::RequestFail);
        } else {
            match response.unwrap().text().await {
                Ok(sip) => return Ok(sip),
                Err(_e) => return Err(JiraError::TextChange)
            }
        }
    }
    
}