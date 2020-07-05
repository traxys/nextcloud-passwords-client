use crate::{create_binding, create_calls, create_details, Error};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

create_calls! {
    PasswordApi where
        Endpoint = "1.0/password",
        Details: Details,
        Type: Password,
        Create: CreatePassword,
        Update: UpdatePassword,
        Error: Error,
        Identifier: PasswordIdentifier,
        Trashed: TrashedIdentifier,
        Criteria: PasswordSearch
    {
        List;
        /// The list action lists all passwords of the user except those in trash and the hidden ones.
        ///
        /// The return value is a list of password objects with the given detail level
        ///
        /// Notes
        ///  - The list will not include trashed passwords
        ///  - The list will not include hidden passwords
        ///  - The list will not include suspended passwords where the folder or a parent folder is in the trash
        pub async fn list(&self, details: Option<Details>) -> Result<Vec<Type>, Error>;

        Get;
        /// The show action lists the properties of a single password.
        ///
        /// The return value is a password object with the given detail level
        ///
        /// Notes
        ///  - This is the only action that can access hidden passwords
        pub async fn get(&self, details: Option<Details>, id: uuid::Uuid) -> Result<Type, Error>;

        Find;
        /// The find action can be used to find all passwords matching the given search criteria.
        ///
        /// The return value is a list of password objects that match the criteria with the given detail level
        ///
        /// Notes
        ///  - The property trashed will be set to false if not present
        ///  - The list will not include hidden passwords
        ///  - The list will not include suspended passwords where the folder or a parent folder is in the trash
        pub async fn find(&self, criteria: Criteria, details: Option<Details>) -> Result<Vec<Type>, Error>;

        Create;
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
        pub async fn create(&self, value: Create) -> Result<Identifier, Error>;

        Update;
        /// The update action creates a new revision of a password with an updated set of attributes.
        ///
        /// Notes
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
        pub async fn update(&self, value: Update) -> Result<Identifier, Error>;

        Delete;
        /// The delete action moves a password to the trash or deletes it completely if it is already in the trash.
        ///
        /// Notes
        ///  - If a password is moved to the trash, the relations to tags will be hidden from the tag, but not the password.
        ///  - If the revision is set, the password will only be deleted if that revision is the current revision.
        ///  Otherwise an "Outdated revision id" error is returned.
        ///  This way, a password is not accidentally deleted instead of trashed if the client is out of sync.
        pub async fn delete(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Trashed, Error>;

        Restore;
        /// The restore action can restore an earlier state of a password.
        ///
        /// Notes
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
        pub async fn restore(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Identifier, Error>;
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

/// Identifies a trashed Password if the revision in `Some`, else identifies a deleted Password
#[derive(Serialize, Deserialize)]
pub struct TrashedIdentifier {
    pub id: uuid::Uuid,
    pub revision: Option<uuid::Uuid>,
}

/// The security status of the password
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
pub enum FolderInfoKind {
    Id(uuid::Uuid),
    Data(crate::folder::Folder),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct FolderInfo(FolderInfoKind);

impl FolderInfo {
    pub fn get(&self) -> &FolderInfoKind {
        &self.0
    }
    pub fn new(id: uuid::Uuid) -> Self {
        Self(FolderInfoKind::Id(id))
    }
}


create_binding! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Password {
        /// User defined label of the password
        pub label: String [create(required) versioned(true) update(required)],
        /// Username associated with the password
        pub username: String [create(optional) versioned(true) update(optional)],
        /// The actual password
        pub password: String [create(required) versioned(true) update(required)],
        /// Url of the website
        pub url: String [create(optional) versioned(true) update(optional)],
        /// Notes for the password. Can be formatted with Markdown
        pub notes: String [create(optional) versioned(true) update(optional)],
        /// Custom fields created by the user.
        #[serde(rename = "customFields")]
        pub custom_fields: String [create(optional) versioned(true) update(optional)],
        /// SHA1 hash of the password
        pub hash: String [create(required) versioned(true) update(required)],
        /// Type of the used client side encryption
        #[serde(rename = "cseType")]
        pub cse_type: String [create(optional) versioned(true) update(optional) search],
        /// UUID of the key used for client side encryption
        #[serde(rename = "cseKey")]
        pub cse_key: String [create(optional) versioned(true) update(optional)],
        /// Type of the used server side encryption
        #[serde(rename = "sseType")]
        pub sse_type: String [versioned(true) search],
        /// Hides the password in list / find actions
        pub hidden: bool [create(optional) versioned(true) update(optional)],
        /// True if the user has marked the password as favorite
        pub favorite: bool [create(optional) versioned(true) update(optional) search],
        /// Unix timestamp when the user last changed the password
        pub edited: i64 [create(optional) versioned(true) update(optional) search],

        /// True if the password is in the trash
        pub trashed: bool [ versioned(true) search],
        /// Unix timestamp when the password was updated
        pub updated: i64 [versioned(true) search],
        /// Name of the client which created this revision
        pub client: String [ versioned(true)],
        /// Security status level of the password
        pub status: SecurityStatus [versioned(true)  search],
        /// Specific code for the current security status
        #[serde(rename = "statusCode")]
        pub status_code: StatusCode [versioned(true)],

        /// The UUID of the password
        pub id: uuid::Uuid [update(required) versioned(false)],
        /// UUID of the current revision
        pub revision: uuid::Uuid [versioned(false)],
        /// UUID of the share if the password was shared by someone else with the user
        pub share: Option<uuid::Uuid> [versioned(false)],
        /// True if the password is shared with other users
        pub shared: bool [versioned(false)],
        /// Specifies if the encrypted properties can be changed. Might be false for shared passwords
        pub editable: bool [versioned(false)],
        /// Unix timestamp when the password was created
        pub created: i64 [search versioned(false)],
        /// Either the UUID of the current folder of the password or the folder model
        pub folder: FolderInfo [create(optional) update(optional) versioned(false)],

        /// Adds the tags property filled with the base model of all tags. Hidden tags are not included in this list if the password is not hidden
        pub tags: Option<Vec<()>> [versioned(false)],
        /// Adds the shares property filled with the base model of all shares with other users. Fills the share property with the base model of the original share if available
        pub shares: Option<Vec<()>> [versioned(false)],
        /// Adds the revisions property which contains all revisions. A revision consists of all properties marked as versioned and its own created property
        pub revisions: Option<Vec<VersionedPassword>> [versioned(false)],
    }
}
