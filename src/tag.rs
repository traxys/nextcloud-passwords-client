use crate::{create_binding, create_details, Color, create_calls};
use serde::{Serialize, Deserialize};

create_calls! {
    TagApi where 
        Endpoint = "1.0/tag",
        Details: Details,
        Type: Tag,
        Create: CreateTag,
        Update: UpdateTag,
        Error: crate::Error,
        Identifier: TagIdentifier,
        Trashed: TrashedIdentifier,
        Criteria: TagSearch 
    {
        List;
        /// The list action lists all tags of the user except those in trash and the hidden ones.
        ///
        /// Notes
        ///  - The list will not include trashed tags
        ///  - The list will not include hidden tags
        pub async fn list(&self, details: Option<Details>) -> Result<Vec<Type>, Error>;

        Get;
        /// The show action lists the properties of a single tag.
        ///
        /// Notes
        ///  - This is the only action that can access hidden tags
        pub async fn get(&self, details: Option<Details>, id: uuid::Uuid) -> Result<Type, Error>;

        Find;
        /// The find action can be used to find all tags matching the given search criteria
        ///
        /// Notes
        ///  - The property trashed will be set to false if not present
        ///  - The list will not include hidden tags
        pub async fn find(&self, criteria: Criteria, details: Option<Details>) -> Result<Vec<Type>, Error>;

        Create;
        /// The create action creates a new tag with the given attributes.
        ///
        /// Notes
        ///  - If the edited argument is "0", missing or in the future, the current time will be used
        pub async fn create(&self, value: Create) -> Result<Identifier, Error>;

        Update;
        /// The update action creates a new revision of a tag with an updated set of attributes.
        ///
        /// Notes
        ///  - If you hide a tag, the tag will be no longer visible in passwords 
        ///  which are not hidden, but the passwords will be visible in the tag
        ///  - If the edited argument is "0" or missing, the timestamp from the last revision will be used
        ///  - If the edited time is in the future, the current time will be used
        pub async fn update(&self, value: Update) -> Result<Identifier, Error>;

        Delete;
        /// The delete action moves a tag to the trash or deletes it completely if it is already in the trash.
        ///
        /// Notes
        ///  - If a tag is moved to the trash, the relation to all passwords which are not in trash will be hidden from the password
        ///  - If a tag is deleted, all relations to passwords are deleted
        ///  - If the revision is set, the tag will only be deleted if that revision is the current revision.
        ///    This way, a tag is not accidentally deleted instead of trashed if the client is out of sync.
        pub async fn delete(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Trashed, Error>;

        Restore;
        /// The restore action can restore an earlier state of a tag.
        ///
        /// Notes
        ///  - If no revision is given and the tag is in trash, it will be removed from trash
        ///  - If no revision is given and the tag is not in trash, nothing is done
        ///  - If a revision is given and the revision is marked as in trash, it will be removed from trash
        ///  - This action will always create a new revision
        ///  - The server side encryption type may change
        ///  - Deleted tags can not be restored
        pub async fn restore(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Identifier, Error>;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TagIdentifier {
    pub id: uuid::Uuid,
    pub revision: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrashedIdentifier {
    pub id: uuid::Uuid,
    pub revision: Option<uuid::Uuid>,
}

create_details! {
    pub struct Details {
        pub revisions: bool,
        pub passwords: bool,
    }
}

create_binding! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Tag {
        pub id: String [update(required) versioned(false)],
        pub label: String [versioned(true) create(required) update(required)],
        pub color: Color [versioned(true) create(required) update(required)],
        pub created: u64 [search versioned(false)],
        pub updated: u64 [versioned(true) search],
        pub edited: u64 [versioned(true) create(optional) update(optional) search],
        pub revision: uuid::Uuid [versioned(true)],
        #[serde(rename = "cseType")]
        pub cse_type: String [versioned(true) create(optional) update(optional) search],
        /// UUID of the key used for client side encryption
        #[serde(rename = "cseKey")]
        pub cse_key: String [versioned(true) create(optional) update(optional)],
        /// Type of the used server side encryption
        #[serde(rename = "sseType")]
        pub sse_type: String [versioned(true) search],
        pub client: String [versioned(true)],
        pub hidden: bool [versioned(true) create(optional) update(optional)],
        pub trashed: bool [versioned(true) search],
        pub favorite: bool [versioned(true) create(optional) update(optional) search],

        pub revisions: Option<Vec<VersionedTag>> [versioned(false)],
    }
}
