use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Event{
    pub timestamp: i64,
    pub webhook_event: String,
    pub project_id: String,
    pub project_name: String,
    pub issue_key: String,
    pub summary: String,
    pub issue_type: String,
    pub assignee: String,
    pub user: String,
    pub changes: String,
    pub comment: String
}