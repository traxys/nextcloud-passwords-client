use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Folder {
    pub id: uuid::Uuid,
    pub label: String,
    pub parent: uuid::Uuid,
    pub created: u64,
    pub updated: u64,
    pub edited: u64,
    pub revision: uuid::Uuid,
    #[serde(rename = "cseType")]
    pub cse_type: String,
    #[serde(rename = "cseKey")]
    pub cse_key: String,
    #[serde(rename = "sseType")]
    pub sse_type: String,
    pub client: String,
    pub hidden: bool,
    pub trashed: bool,
    pub favorite: bool,
}
