use crate::{create_binding, create_details, password, AuthenticatedApi, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: figure out how to do (owner, receiver)

pub struct ShareApi<'a> {
    pub(crate) api: &'a AuthenticatedApi,
}

impl<'a> ShareApi<'a> {
    /// This commands returns an array of users that the current user can share with.
    ///
    /// Notes
    ///  - This command will fail if sharing is disabled
    ///  - The limit can not be less than 5 or more than 256
    ///  - This api endpoint has a rate limit of 45 requests per minute
    pub async fn partners(
        &self,
        search: Option<String>,
        limit: Option<u64>,
    ) -> Result<Vec<Partner>, Error> {
        #[derive(Serialize, Deserialize)]
        struct Request {
            #[serde(skip_serializing_if = "Option::is_none")]
            search: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            limit: Option<u64>,
        }
        let req = Request { search, limit };
        let ret: Vec<HashMap<uuid::Uuid, String>> = if req.search.is_none() {
            self.api
                .passwords_get("/api/1.0/share/partners", req)
                .await?
        } else {
            self.api
                .passwords_post("/api/1.0/share/partners", req)
                .await?
        };
        Ok(ret
            .into_iter()
            .map(|partner| {
                partner
                    .into_iter()
                    .map(|(user_id, display_name)| Partner {
                        user_id,
                        display_name,
                    })
                    .next()
            })
            .filter_map(std::convert::identity)
            .collect())
    }

    /// The create action creates a new share with the given attributes.
    ///
    /// Notes
    ///  - This action will fail if the password is hidden or the CSE does not support sharing
    ///  - You can not share a password with the same user more than once
    ///  - This command will fail if sharing is disabled
    pub async fn create(&self, create: CreateShare) -> Result<uuid::Uuid, Error> {
        #[derive(Serialize, Deserialize)]
        struct Resp {
            id: uuid::Uuid,
        }
        let resp: Resp = self
            .api
            .passwords_post("/api/1.0/share/create", create)
            .await?;
        Ok(resp.id)
    }

    /// The update action changes the properties of an existing share.
    ///
    /// Notes
    ///  - You can only edit a share if it is owned by the user
    ///  - This command will fail if sharing is disabled
    pub async fn update(&self, update: UpdateShare) -> Result<uuid::Uuid, Error> {
        #[derive(Serialize, Deserialize)]
        struct Resp {
            id: uuid::Uuid,
        }
        let resp: Resp = self
            .api
            .passwords_post("/api/1.0/share/update", update)
            .await?;
        Ok(resp.id)
    }

    /// The delete action deletes a share.
    ///
    /// Notes
    ///  - You can only delete shares owned by the user.
    ///  - If you want to delete a share where the current user is the receiver, you need to delete the password instead
    ///  - This action still works if sharing has been disabled
    pub async fn delete(&self, share_id: uuid::Uuid) -> Result<uuid::Uuid, Error> {
        #[derive(Serialize, Deserialize)]
        struct Id {
            id: uuid::Uuid,
        }
        let resp: Id = self
            .api
            .passwords_post("/api/1.0/share/delete", Id { id: share_id })
            .await?;
        Ok(resp.id)
    }

    /// The list action lists all shares with the user as owner or receiver.
    ///
    /// Notes
    ///  - This action still works if sharing has been disabled
    pub async fn list(&self, details: Option<Details>) -> Result<Vec<Share>, Error> {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct DetailsStr {
            #[serde(skip_serializing_if = "Option::is_none")]
            details: Option<String>,
        }
        self.api
            .passwords_post(
                "1.0/share/list",
                DetailsStr {
                    details: details.map(|d| d.to_string()),
                },
            )
            .await
    }

    /// The show action lists the properties of a single share.
    ///
    /// Notes
    ///  - This action still works if sharing has been disabled
    pub async fn get(&self, details: Option<Details>, id: uuid::Uuid) -> Result<Share, Error> {
        #[derive(Serialize, Deserialize)]
        struct Show {
            id: uuid::Uuid,
            #[serde(skip_serializing_if = "Option::is_none")]
            details: Option<String>,
        }
        let request = Show {
            id,
            details: details.map(|d| d.to_string()),
        };
        self.api.passwords_post("1.0/share/show", request).await
    }
    
    /// The find action can be used to find all shares matching the given search criteria
    ///
    /// Notes
    ///  - This action still works if sharing has been disabled
    pub async fn find(
        &self,
        criteria: ShareSearch,
        details: Option<Details>,
    ) -> Result<Vec<Share>, Error> {
        #[derive(Serialize)]
        struct Request {
            criteria: ShareSearch,
            #[serde(skip_serializing_if = "Option::is_none")]
            details: Option<String>,
        }
        let request = Request {
            criteria,
            details: details.map(|d| d.to_string()),
        };
        self.api.passwords_post("1.0/share/find", request).await
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateShare {
    password: uuid::Uuid,
    receiver: uuid::Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    ty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires: Option<Option<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    editable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shareable: Option<bool>,
}
impl CreateShare {
    pub fn new(password_id: uuid::Uuid, receiver_id: uuid::Uuid) -> Self {
        CreateShare {
            password: password_id,
            receiver: receiver_id,
            ty: None,
            expires: None,
            editable: None,
            shareable: None,
        }
    }
    /// The type of the share
    pub fn share_type(self, ty: String) -> Self {
        Self {
            ty: Some(ty),
            ..self
        }
    }
    /// Unix timestamp when the share will expire
    pub fn expires(self, expires: Option<u64>) -> Self {
        Self {
            expires: Some(expires),
            ..self
        }
    }
    /// Whether or not the receiver can edit the password
    pub fn editable(self, editable: bool) -> Self {
        Self {
            editable: Some(editable),
            ..self
        }
    }
    /// Whether or not the receiver can share the password
    pub fn shareable(self, shareable: bool) -> Self {
        Self {
            shareable: Some(shareable),
            ..self
        }
    }
}

#[derive(Debug)]
pub struct Partner {
    pub user_id: uuid::Uuid,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    pub id: uuid::Uuid,
    pub name: String,
}

create_details! {
    pub struct Details {
        pub password: bool,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PasswordInfoKind {
    Id(uuid::Uuid),
    Data(password::Password),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct PasswordInfo(PasswordInfoKind);
impl PasswordInfo {
    pub fn get(&self) -> &PasswordInfoKind {
        &self.0
    }
    pub fn new(id: uuid::Uuid) -> Self {
        Self(PasswordInfoKind::Id(id))
    }
}

create_binding! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Share {
        pub id: uuid::Uuid [update(required) versioned(false)],
        pub created: u64 [search versioned(false)],
        pub updated: u64 [search versioned(false)],
        pub expires: Option<u64> [update(optional) search versioned(false)],
        pub editable: bool [update(optional) search versioned(false)],
        pub shareable: bool [update(optional) search versioned(false)],
        #[serde(rename = "updatePending")]
        pub update_pending: bool [ versioned(false)],
        pub password: PasswordInfo [ versioned(false)],
        pub owner: Person [versioned(false)],
        pub receiver: Person [versioned(false)],
    }
}
