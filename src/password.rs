use crate::{create_binding, create_details, Error};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Password related actions
pub struct PasswordApi<'a> {
    pub(crate) api: &'a crate::AuthenticatedApi,
}

impl<'a> PasswordApi<'a> {
    /// The show action lists the properties of a single password.
    ///
    /// The return value is a password object with the given detail level
    ///
    /// Notes
    ///  - This is the only action that can access hidden passwords
    pub async fn get(&self, details: Option<Details>, id: &str) -> Result<Password, Error> {
        #[derive(Serialize, Deserialize)]
        struct ShowPassword<'a> {
            id: &'a str,
            details: String,
        }
        let request = ShowPassword {
            id,
            details: details.unwrap_or_else(Default::default).to_string(),
        };
        self.api
            .passwords_post("1.0/password/show", request)
            .await
            .map_err(Into::into)
    }

    /// The list action lists all passwords of the user except those in trash and the hidden ones.
    ///
    /// The return value is a list of password objects with the given detail level
    ///
    /// Notes
    ///  - The list will not include trashed passwords
    ///  - The list will not include hidden passwords
    ///  - The list will not include suspended passwords where the folder or a parent folder is in the trash
    pub async fn list(&self, details: Details) -> Result<Vec<Password>, Error> {
        #[derive(Serialize, Deserialize)]
        struct DetailsStr {
            details: String,
        }
        Ok(self
            .api
            .passwords_post(
                "1.0/password/list",
                DetailsStr {
                    details: details.to_string(),
                },
            )
            .await?)
    }

    /// The create action creates a new password with the given attributes.
    ///
    /// Notes
    ///  - If the password is not hidden and should be created in a hidden folder, it will be created in the base folder instead
    ///  - If the folder uuid is invalid or does not exist, the base folder uuid will be used instead
    ///  - If the edited argument is "0" or missing, the current time will be used
    ///  - If the edited time is in the future, the current time will be used
    ///  - If the cseType is set to "none", the hash will be calculated on the server
    ///  - If the tags argument contains invalid tag ids, they will be ignored
    ///  - You can assign hidden tags to a not hidden password, but they will not be visible.
    ///  - Therefore another client might remove the tag by accident
    pub async fn create(&self, password: CreatePassword) -> Result<PasswordIdentifier, Error> {
        self.api
            .passwords_post("1.0/password/create", password)
            .await
            .map_err(Into::into)
    }

    /// The update action creates a new revision of a password with an updated set of attributes.
    ///
    /// Notes
    ///
    ///  - If the password is not editable any change to the encrypted properties, the cseType and the hash will be ignored.
    ///  - If the password is shared you can only use cse types which support sharing
    ///  - If the password is shared you can not hide the password
    ///  - If the password is not hidden and should be moved to a hidden folder, it will be moved to the base folder instead
    ///  - If the password has tags and you want to remove all tags, you need to submit an array with one invalid tag id
    ///  - If the folder uuid is invalid or does not exist, the base folder uuid will be used instead
    ///  - If the edited argument is "0" or missing, the timestamp from the last revision will be used
    ///  - If the edited time is in the future, the current time will be used
    ///  - If the hash has not changed, the edited field from the last revision will be used
    ///  - If the cseType is set to "none", the hash will be calculated on the server
    ///  - If the tags argument is empty or missing, no changes will be made
    ///  - If the tags argument contains invalid tag ids, they will be ignored
    ///  - You can assign hidden tags to a not hidden password, but they will not be visible.
    ///  - Therefore another client might remove the tag by accident
    pub async fn update(&self, updates: UpdatePassword) -> Result<PasswordIdentifier, Error> {
        self.api
            .passwords_post("1.0/password/update", updates)
            .await
            .map_err(Into::into)
    }

    /// The find action can be used to find all passwords matching the given search criteria.
    ///
    /// The return value is a list of password objects that match the criteria with the given detail level
    ///
    /// Notes
    ///  - The property trashed will be set to false if not present
    ///  - The list will not include hidden passwords
    ///  - The list will not include suspended passwords where the folder or a parent folder is in the trash
    pub async fn find(
        &self,
        criteria: PasswordSearch,
        details: Details,
    ) -> Result<Vec<Password>, Error> {
        #[derive(Serialize)]
        struct Request {
            criteria: PasswordSearch,
            details: String,
        }
        let request = Request {
            criteria,
            details: details.to_string(),
        };
        self.api
            .passwords_post("1.0/password/find", request)
            .await
            .map_err(Into::into)
    }

    /// The restore action can restore an earlier state of a password.
    ///
    /// Notes
    ///
    ///  - If no revision is given and the password is in trash, it will be removed from trash
    ///  - If no revision is given and the password is not in trash, nothing is done
    ///  - If a revision is given and the revision is marked as in trash, it will be removed from trash
    ///  - If a revision is given that does not belong to the model, a "Invalid revision id" error will be returned.
    ///  - The action will fail if the password is shared but the revision to restore does not meet the requirements for sharing
    ///  - This action will always create a new revision
    ///  - The server side encryption type may change
    ///  - If the folder does not exist anymore, it will be moved to the base folder
    ///  - Tag relations can not be restored
    ///  - Deleted passwords can not be restored
    pub async fn restore(
        &self,
        id: uuid::Uuid,
        revision: Option<uuid::Uuid>,
    ) -> Result<PasswordIdentifier, Error> {
        #[derive(Serialize)]
        struct Request {
            id: uuid::Uuid,
            revision: Option<uuid::Uuid>,
        }
        self.api
            .passwords_patch("1.0/password/restore", Request { id, revision })
            .await
            .map_err(Into::into)
    }

    /// The delete action moves a password to the trash or deletes it completely if it is already in the trash.
    ///
    /// Notes
    ///
    ///  - If a password is moved to the trash, the relations to tags will be hidden from the tag, but not the password.
    ///  - If the revision is set, the password will only be deleted if that revision is the current revision.
    ///  Otherwise an "Outdated revision id" error is returned. 
    ///  This way, a password is not accidentally deleted instead of trashed if the client is out of sync.
    pub async fn delete(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<TrashedIdentifier, Error> {
        #[derive(Serialize)]
        struct Request {
            id: uuid::Uuid,
            revision: Option<uuid::Uuid>,
        }
        self.api
            .passwords_delete("1.0/password/delete", Request { id, revision })
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

/// Identifies a password by it's id and revision
#[derive(Serialize, Deserialize)]
pub struct PasswordIdentifier {
    pub id: uuid::Uuid,
    pub revision: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct TrashedIdentifier {
    pub id: uuid::Uuid,
    pub revision: Option<uuid::Uuid>,
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
        pub cse_type: String [create(optional) versioned update(optional) search],
        /// UUID of the key used for client side encryption
        #[serde(rename = "cseKey")]
        pub cse_key: String [create(optional) versioned update(optional)],
        /// Type of the used server side encryption
        #[serde(rename = "sseType")]
        pub sse_type: String [versioned search],
        /// Hides the password in list / find actions
        pub hidden: bool [create(optional) versioned update(optional)],
        /// True if the user has marked the password as favorite
        pub favorite: bool [create(optional) versioned update(optional) search],
        /// Unix timestamp when the user last changed the password
        pub edited: i64 [create(optional) versioned update(optional) search],

        /// True if the password is in the trash
        pub trashed: bool [ versioned search],
        /// Unix timestamp when the password was updated
        pub updated: i64 [versioned search],
        /// Name of the client which created this revision
        pub client: String [ versioned],
        /// Security status level of the password
        pub status: SecurityStatus [versioned search],
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
        pub created: i64 [search],
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
