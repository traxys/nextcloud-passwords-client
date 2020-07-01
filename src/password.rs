use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use serde_json::Value;
use crate::create_binding;

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum SecurityStatus {
    Ok = 0,
    UserRulesViolated = 1,
    Breached = 2,
}

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
#[serde(untagged)]
pub enum FolderInfo {
    Id(uuid::Uuid),
    Data(crate::folder::Folder),
}

create_binding! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Password {
        pub label: String [writable versioned],
        pub username: String [writable versioned],
        pub password: String [writable versioned],
        pub url: String [writable versioned],
        pub notes: String [writable versioned],
        #[serde(rename = "customFields")]
        pub custom_fields: String [writable versioned],
        pub hash: String [writable versioned],
        #[serde(rename = "cseType")]
        pub cse_type: String [writable versioned],
        #[serde(rename = "cseKey")]
        pub cse_key: String [writable versioned],
        #[serde(rename = "sseType")]
        pub sse_type: String [writable versioned],
        pub hidden: bool [writable versioned],
        pub favorite: bool [writable versioned],
        pub edited: i64 [writable versioned],

        pub trashed: bool [ versioned],
        pub updated: i64 [ versioned],
        pub client: String [ versioned],
        pub status: SecurityStatus [versioned],
        #[serde(rename = "statusCode")]
        pub status_code: StatusCode [versioned],

        pub id: uuid::Uuid [],
        pub revision: uuid::Uuid [],
        pub share: Option<uuid::Uuid> [],
        pub shared: bool [],
        pub editable: bool [],
        pub created: i64 [],
        pub folder: FolderInfo [],

        pub tags: Option<Vec<()>> [],
        pub shares: Option<Vec<()>> [],
        pub revisions: Option<Vec<VersionedPassword>> [],
    }
}

