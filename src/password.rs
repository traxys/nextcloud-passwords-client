use crate::{create_binding, create_details, Error};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Password related actions
pub struct PasswordApi<'a> {
    pub(crate) api: &'a crate::AuthenticatedApi,
}

impl<'a> PasswordApi<'a> {
    /// Get a single password with it's ID
    pub async fn get_password(
        &self,
        details: Option<Details>,
        id: &str,
    ) -> Result<Password, Error> {
        #[derive(Serialize, Deserialize)]
        struct ShowPassword<'a> {
            id: &'a str,
            details: String,
        }
        let request = ShowPassword {
            id,
            details: details.unwrap_or_else(Default::default).to_string(),
        };
        self.api.passwords_post("1.0/password/show", request)
            .await
            .map_err(Into::into)
    }
    /// List all the passwords known for this user
    pub async fn list_passwords(
        &self,
        details: Details,
    ) -> Result<Vec<Password>, Error> {
        #[derive(Serialize, Deserialize)]
        struct Details {
            details: String,
        }
        Ok(self.api
            .passwords_post(
                "1.0/password/list",
                Details {
                    details: details.to_string(),
                },
            )
            .await?)
    }
    /// Create a password
    pub async fn create_password(
        &self,
        password: CreatePassword,
    ) -> Result<PasswordCreated, Error> {
        self.api.passwords_post("1.0/password/create", password)
            .await
            .map_err(Into::into)
    }
}

create_details! {
    pub struct Details {
        pub revisions: bool,
        pub folder: bool,
        pub tags: bool,
        pub shares: bool,
    }
}

#[derive(Serialize, Deserialize)]
pub struct PasswordCreated {
    pub id: uuid::Uuid,
    pub revision: uuid::Uuid,
}

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
        /// User defined label of the password
        pub label: String [create(required) versioned update(required)],
        /// Username associated with the password
        pub username: String [create(optional) versioned update(optional)],
        /// The actual password
        pub password: String [create(required) versioned update(required)],
        /// Url of the website
        pub url: String [create(optional) versioned update(optional)],
        /// Notes for the password. Can be formatted with Markdown
        pub notes: String [create(optional) versioned update(optional)],
        /// Custom fields created by the user.
        #[serde(rename = "customFields")]
        pub custom_fields: String [create(optional) versioned update(optional)],
        /// SHA1 hash of the password
        pub hash: String [create(required) versioned update(required)],
        /// Type of the used client side encryption
        #[serde(rename = "cseType")]
        pub cse_type: String [create(optional) versioned update(optional)],
        /// UUID of the key used for client side encryption
        #[serde(rename = "cseKey")]
        pub cse_key: String [create(optional) versioned update(optional)],
        /// Type of the used server side encryption
        #[serde(rename = "sseType")]
        pub sse_type: String [versioned],
        /// Hides the password in list / find actions
        pub hidden: bool [create(optional) versioned update(optional)],
        /// True if the user has marked the password as favorite
        pub favorite: bool [create(optional) versioned update(optional)],
        /// Unix timestamp when the user last changed the password
        pub edited: i64 [create(optional) versioned update(optional)],

        /// True if the password is in the trash
        pub trashed: bool [ versioned],
        /// Unix timestamp when the password was updated
        pub updated: i64 [ versioned],
        /// Name of the client which created this revision
        pub client: String [ versioned],
        /// Security status level of the password
        pub status: SecurityStatus [versioned],
        /// Specific code for the current security status
        #[serde(rename = "statusCode")]
        pub status_code: StatusCode [versioned],

        /// The UUID of the password
        pub id: uuid::Uuid [update(required)],
        /// UUID of the current revision
        pub revision: uuid::Uuid [],
        /// UUID of the share if the password was shared by someone else with the user
        pub share: Option<uuid::Uuid> [],
        /// True if the password is shared with other users
        pub shared: bool [],
        /// Specifies if the encrypted properties can be changed. Might be false for shared passwords
        pub editable: bool [],
        /// Unix timestamp when the password was created
        pub created: i64 [],
        /// Either the UUID of the current folder of the password or the folder model
        pub folder: FolderInfo [],

        /// Adds the tags property filled with the base model of all tags. Hidden tags are not included in this list if the password is not hidden
        pub tags: Option<Vec<()>> [],
        /// Adds the shares property filled with the base model of all shares with other users. Fills the share property with the base model of the original share if available
        pub shares: Option<Vec<()>> [],
        /// Adds the revisions property which contains all revisions. A revision consists of all properties marked as versioned and its own created property
        pub revisions: Option<Vec<VersionedPassword>> [],
    }
}
