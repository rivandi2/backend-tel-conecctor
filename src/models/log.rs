use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Log{
    pub event: String,
    pub status: String,
    pub attempt: i32,
    pub time: String,
}