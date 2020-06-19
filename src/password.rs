use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum StatusCode {
    #[serde(rename = "GOOD")]
    Good,
    #[serde(rename = "OUTDATED")]
    Outdated,
    #[serde(rename = "DUPLICATE")]
    Duplicate,
    #[serde(rename = "BREACHED")]
    Breached,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Password {
    pub id: uuid::Uuid,
    pub label: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    #[serde(rename = "customFields")]
    pub custom_fields: String,
    pub status: u8,
    #[serde(rename = "statusCode")]
    pub status_code: StatusCode,
    pub hash: String,
    pub folder: uuid::Uuid,
}
