use crate::{create_binding, create_details, AuthenticatedApi, Error};
use serde::{Deserialize, Serialize};

pub struct FolderApi<'a> {
    pub(crate) api: &'a AuthenticatedApi,
}

impl<'a> FolderApi<'a> {
    /// The create action creates a new folder with the given attributes.
    ///
    /// Notes
    ///  - If the uuid of the parent folder is invalid or does not exist, the base folder uuid will be used instead
    ///  - If the folder is not hidden and should be created in a hidden folder, it will be created in the base folder instead
    ///  - If the edited argument is "0", missing or in the future, the current time will be used
    pub async fn create(&self, folder: CreateFolder) -> Result<FolderIdentifier, Error> {
        self.api
            .passwords_post("1.0/folder/create", folder)
            .await
            .map_err(Into::into)
    }

    /// The update action creates a new revision of a folder with an updated set of attributes.
    ///
    /// Notes
    ///  - If the uuid of the parent folder is invalid or does not exist, the base folder uuid will be used instead
    ///  - If the folder is not hidden and should be moved to a hidden parent folder, it will be moved to the base folder instead
    ///  - If you hide a folder, all folders and passwords in it will be hidden as well
    ///  - If you unhide a folder no change to the folders and passwords in it will be made and they will remain hidden
    ///  - If the edited argument is "0" or missing, the timestamp from the last revision will be used
    ///  - If the edited time is in the future, the current time will be used
    pub async fn update(&self, folder: UpdateFolder) -> Result<FolderIdentifier, Error> {
        self.api
            .passwords_post("1.0/folder/update", folder)
            .await
            .map_err(Into::into)
    }

    /// The delete action moves a folder and its content to the trash or deletes it completely if it is already in the trash.
    ///
    /// Notes
    ///  - If a folder is moved to the trash, all passwords and folders in it will be suspended and hidden from list and find actions
    ///  - If a folder is moved to the trash, the relations between tags and passwords in the folder will be hidden from the tag, but not the password
    ///  - If a folder is deleted, all passwords and folders in it will be deleted as well
    ///  - If the revision is set, the folder will only be deleted if that revision is the current revision. 
    ///  This way, a folder is not accidentally deleted instead of trashed if the client is out of sync.
    pub async fn delete(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<TrashedIdentifier, Error> {
        #[derive(Serialize)]
        struct Request {
            id: uuid::Uuid,
            revision: Option<uuid::Uuid>,
        }
        self.api
            .passwords_delete("1.0/folder/delete", Request { id, revision })
            .await
            .map_err(Into::into)
    }

    /// The restore action can restore an earlier state of a folder.
    ///
    /// Notes
    ///  - If no revision is given and the folder is in trash, it will be removed from trash
    ///  - If no revision is given and the folder is not in trash, nothing is done
    ///  - If a revision is given and the revision is marked as in trash, it will be removed from trash
    ///  - This action will always create a new revision
    ///  - The server side encryption type may change
    ///  - If the parent folder does not exist anymore, it will be moved to the base folder
    ///  - Deleted folders can not be restored
    pub async fn restore(
        &self,
        id: uuid::Uuid,
        revision: Option<uuid::Uuid>,
    ) -> Result<FolderIdentifier, Error> {
        #[derive(Serialize)]
        struct Request {
            id: uuid::Uuid,
            revision: Option<uuid::Uuid>,
        }
        self.api
            .passwords_patch("1.0/folder/restore", Request { id, revision })
            .await
            .map_err(Into::into)
    }

    /// The show action lists the properties of a single folder.
    ///
    /// Notes
    ///  - This is the only action that can access hidden folders
    pub async fn get(&self, details: Option<Details>, id: &str) -> Result<Folder, Error> {
        #[derive(Serialize, Deserialize)]
        struct ShowFolder<'a> {
            id: &'a str,
            details: String,
        }
        let request = ShowFolder {
            id,
            details: details.unwrap_or_else(Default::default).to_string(),
        };
        self.api
            .passwords_post("1.0/folder/show", request)
            .await
            .map_err(Into::into)
    }

    /// The list action lists all folders of the user except those in trash and the hidden ones.
    ///
    /// Notes
    ///  - The list will not include trashed folders
    ///  - The list will not include hidden folders
    ///  - The list will not include suspended folders where a parent folder is in the trash
    pub async fn list(&self, details: Details) -> Result<Vec<Folder>, Error> {
        #[derive(Serialize, Deserialize)]
        struct DetailsStr {
            details: String,
        }
        Ok(self
            .api
            .passwords_post(
                "1.0/folder/list",
                DetailsStr {
                    details: details.to_string(),
                },
            )
            .await?)
    }

    /// The find action can be used to find all folders matching the given search criteria
    ///
    /// Notes
    ///  - The property trashed will be set to false if not present
    ///  - The property parent is only supported in 2019.5.0 and later
    ///  - The list will not include hidden folders
    ///  - The list will not include suspended folders where a parent folder is in the trash
    pub async fn find(
        &self,
        criteria: FolderSearch,
        details: Details,
    ) -> Result<Vec<Folder>, Error> {
        #[derive(Serialize)]
        struct Request {
            criteria: FolderSearch,
            details: String,
        }
        let request = Request {
            criteria,
            details: details.to_string(),
        };
        self.api
            .passwords_post("1.0/folder/find", request)
            .await
            .map_err(Into::into)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FolderIdentifier {
    pub id: uuid::Uuid,
    pub revision: uuid::Uuid,
}
#[derive(Serialize, Deserialize)]
pub struct TrashedIdentifier {
    pub id: uuid::Uuid,
    pub revision: Option<uuid::Uuid>,
}

create_details! {
    pub struct Details {
        pub revisions: bool,
        //pub parent: bool,
        pub folders: bool,
        pub passwords: bool,
        pub tags: bool,
    }
}

create_binding! {
#[derive(Serialize, Deserialize, Debug)]
pub struct Folder {
    pub id: uuid::Uuid [update(required)],
    pub label: String [versioned create(required) update(required)],
    pub parent: uuid::Uuid [versioned create(optional) update(optional) search],
    pub created: u64 [search],
    pub updated: u64 [versioned search],
    pub edited: u64 [versioned update(optional)],
    pub revision: uuid::Uuid [versioned],
    #[serde(rename = "cseType")]
    pub cse_type: String [versioned create(optional) update(optional) search],
    #[serde(rename = "cseKey")]
    pub cse_key: String [versioned create(optional) update(optional)],
    #[serde(rename = "sseType")]
    pub sse_type: String [versioned search],
    pub client: String [versioned],
    pub hidden: bool [versioned create(optional) update(optional)],
    pub trashed: bool [versioned search],
    pub favorite: bool [versioned create(optional) update(optional) search],

    pub revisions: Option<Vec<VersionedFolder>> [],
    pub folders: Option<Vec<Folder>> [],
    pub passwords: Option<Vec<crate::password::Password>> [],
}
}
