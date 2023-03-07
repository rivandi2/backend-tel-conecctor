use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event{
    pub id: String,
    team_id: String,
    webhook_id: String,
    source_id: String,
    destination_id: String,
    cli_id: Option<String>,
    request_id: String, 
    event_data_id: String, 
    attempts: i32,
    status: Option<String>,
    response_status	: Option<i32>,
    error_code: Option<String>,
    last_attempt_at: Option<String>,
    next_attempt_at: Option<String>,
    sucessfull_at: Option<String>,
    updated_at: String,
    pub created_at: String,
} 

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination{
    order_by: String,
    dir: String,
    limit: i32,
    next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookdeckEvents {
    pagination: Pagination,
    count: i32,
    pub models: Vec<Event>
}