use std::{env, borrow::Borrow};
// use futures::StreamExt;
use serde::{Serialize, Deserialize};
use serde_json;
use thiserror::Error;
use dotenv::dotenv;
use mongodb::{Client, options::{ClientOptions, ResolverConfig}, Collection};

mod project_model;
use project_model::Project;
use project_model::Event;


#[derive(Error, Debug)]
pub enum JiraError {
    #[error("Placehold error")] ProjectFound(#[from] reqwest::Error),
    #[error("Project tidak dapat ditemukan")] ProjectChange,
    #[error("Event tidak dapat ditemukan")] EventChange,
    #[error("Tidak dapat diubah ke JSON text")] TextChange,
    #[error("Tidak bisa masukin ke vector")] VectorFail(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Klien 
{
    pub reqwest: reqwest::Client,
    pub mongodb: mongodb::Client

}

impl Klien
{
    pub fn new() -> Self
    {
        return Self{
            reqwest: reqwest::Client::new(),
            mongodb: futures::executor::block_on(mongodb::Client::with_uri_str("mongodb+srv://rivandi:Z0BDxmHQc9k237ES@cluster0.kixsqjq.mongodb.net/?retryWrites=true&w=majority")).unwrap()
        }
    }
    // env::var("MONGO_URI")

    pub async fn get_projects(&self) -> Result<Vec<Project>, JiraError>{
        let response = self.reqwest
            .get("https://telkomdevelopernetwork.atlassian.net/rest/api/2/project")
            .basic_auth(env::var("USER_NAME").expect("USER_NAME not found"), Some(env::var("PASSWORD").expect("PASSWORD not found")))
            .send()
            .await;
        if response.is_err(){
            return Err(JiraError::ProjectChange);
        } else 
            {
                let text = response.unwrap()
                    .text()
                    .await;
                if text.is_err(){
                    return Err(JiraError::TextChange);
                }  
                else {
                    println!("{:?}", text);
                    let list: Vec<Project> = serde_json::from_str(&text.unwrap()).unwrap();
                    return Ok(list) 
                }      
            }
    }

    pub async fn get_events(&self) -> Result<Vec<Event>, JiraError>{
        let response = self.reqwest
            .get("https://telkomdevelopernetwork.atlassian.net/rest/api/3/events")
            .basic_auth(env::var("USER_NAME").expect("USER_NAME not found"), Some(env::var("PASSWORD").expect("PASSWORD not found")))
            .send()
            .await;
        if response.is_err(){
            return Err(JiraError::EventChange);
        } else 
            {
                let text = response.unwrap()
                    .text()
                    .await;
                if text.is_err(){
                    return Err(JiraError::TextChange);
                }  
                else {
                    let list: Vec<Event> = serde_json::from_str(&text.unwrap()).unwrap();
                    return Ok(list) 
                }      
            }
    }

    pub async fn add_projects(&self) 
    // where
    // T: Serialize,
    // &Project: Borrow<T>,
    // 
    {
    
        let db = self.mongodb.database("telcon");
        println!("Masuk2");
        for collection_name in db.list_collection_names(None).await {
            println!("Here {:?}", collection_name);
        }
        
    }



}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let klien = Klien::new();
    // let project = klien.get_projects().await;
    // let event = klien.get_events().await;
    // println!("Intial ok");
    // let tes = klien.add_projects(project.unwrap()).await;

    let tes = klien.add_projects().await;
  

    // print project
    // let mut i = 1;
    // let _tes = match project{
    //     Ok(val) => {
    //         for pro in val {
    //             println!("{}.  {} - {} [{}]", i, pro.id, pro.name, pro.key);
    //             i+=1;
    //         }  
    //     },
    //     Err(e) => println!("{}", e)
    // };

    // //print event
    // println!("");
    // let mut j = 1;
    // let _tes = match event{
    //     Ok(val) => {
    //         for ev in val {
    //             println!("{}. {} - {}", j, ev.id, ev.name);
    //             j+=1;
    //         }  
    //     },
    //     Err(e) => println!("{}", e)
    // };
}
