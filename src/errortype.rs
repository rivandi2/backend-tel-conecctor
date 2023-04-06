use thiserror::Error;

#[derive(Error, Debug)]
pub enum JiraError {
    #[error("Placehold error")] ProjectFound(#[from] reqwest::Error),
    #[error("Request unsuccessfull")] RequestFail,
    #[error("Failed to convert to JSON text")] TextChange,
    #[error("Insert to vector unsuccessfull")] VectorFail(#[from] serde_json::Error),
    #[error("{0}")] ErrorMessage(String),
    #[error("Incorect email address")] EmailError,
    #[error("Incorect api key")] ApiKeyError,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConnectorError {
    #[error("Connector not found")] ConNotFound,
    #[error("Connector name already exist")] ConCreateExist,
    #[error("Cannot update to new name [Connector name already exist]")] ConUpdateExist,
    #[error("Connectors list is empty")] ConEmpty,
    #[error("Bot token invalid")] TokenInval,
    #[error("Chatid invalid [Bot not invited to chatid]")] ChatidInval,
    #[error("Bucket error {0}")] RusError(String),
    #[error("Log not found")] LogNotFound
}
