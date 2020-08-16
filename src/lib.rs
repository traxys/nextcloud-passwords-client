use reqwest::Client;
use serde::{Deserialize, Serialize};
pub use url::Url;

/// Data types to interract with the folder API. Check [FolderApi](folder::FolderApi) for the
/// available actions. You can also check the [HTTP
/// API](https://git.mdns.eu/nextcloud/passwords/wikis/Developers/Api/Folder-Api)
pub mod folder;
/// Data types and builders to interact with the passwords API. Check
/// [PasswordApi](password::PasswordApi) for the available actions. You can also check the [HTTP
/// API](https://git.mdns.eu/nextcloud/passwords/wikis/Developers/Api/Password-Api)
pub mod password;
/// Actions available for the service API. Check [ServiceApi](service::ServiceApi) for more
/// information. You can also check the [HTTP
/// API](https://git.mdns.eu/nextcloud/passwords/wikis/Developers/Api/Service-Api)
pub mod service;
/// Data types, helpers and builders to interact with the settings API. Check
/// [SettingsApi](settings::SettingsApi) for the available actions. You can also check the [HTTP
/// API](https://git.mdns.eu/nextcloud/passwords/wikis/Developers/Api/Settings-Api)
pub mod settings;
/// Data types, helpers and builders to interact with the share API. Check
/// [ShareApi](share::ShareApi) for the available actions. You can also check the [HTTP
/// API](https://git.mdns.eu/nextcloud/passwords/wikis/Developers/Api/Share-Api) for more
/// information.
pub mod share;
/// Data types, helpers and builders to interact with the tag API. Check
/// [TagApi](tag::TagApi) for the available actions. You can also check the [HTTP
/// API](https://git.mdns.eu/nextcloud/passwords/wikis/Developers/Api/Tag-Api)
pub mod tag;
/// Data types and helpers to access the Token API. Check [TokenApi](token::TokenApi) for the
/// available actions. You can also check the [HTTP
/// API](https://git.mdns.eu/nextcloud/passwords/wikis/Developers/Api/Token-Api)
pub mod token;

// TODO: sort the session required methods from the non-session required

mod utils;
pub use utils::{QueryKind, SearchQuery};

mod private {
    pub trait Sealed {}

    impl Sealed for super::settings::UserSettings {}
    impl Sealed for super::settings::ServerSettings {}
    impl Sealed for super::settings::ClientSettings {}
}

#[derive(Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
impl std::fmt::Display for Color {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "#{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue
        ))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = Color;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "an hex color of the form #abcdef as a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.len() == 7 {
                    if !value.starts_with('#') {
                        Err(E::custom("expected the color to start with `#`"))
                    } else {
                        let mut result = [0u8; 3];
                        hex::decode_to_slice(value.trim_start_matches('#'), &mut result).map_err(
                            |e| E::custom(format!("Could not parse hex string: {:?}", e)),
                        )?;
                        Ok(Color {
                            red: result[0],
                            green: result[1],
                            blue: result[2],
                        })
                    }
                } else {
                    Err(E::custom(format!(
                        "Expected a string of length 7, got length: {}",
                        value.len()
                    )))
                }
            }
        }

        deserializer.deserialize_str(StrVisitor)
    }
}

/// Errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error in communicating with the API")]
    ApiError(#[from] reqwest::Error),
    #[error("could not connect to the passwords API")]
    ConnectionFailed,
    #[error("could not cleanly disconnect from the passwords API")]
    DisconnectionFailed,
    #[error("last shutdown time is in the future")]
    TimeError(#[from] std::time::SystemTimeError),
    #[error("setting was not valid in this context")]
    InvalidSetting,
    #[error("serde error")]
    Serde(#[from] serde_json::Error),
    #[error("endpoint error: {}", .0.message)]
    EndpointError(EndpointError),
    #[error("error in the login flow: request returned {0}")]
    LoginFlowError(u16),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointError {
    status: String,
    id: u64,
    message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum EndpointResponse<T> {
    Error(EndpointError),
    Success(T),
}

/// Represent how to first connect to a nextcloud instance
/// The best way to obtain some is using [Login flow
/// v2](https://docs.nextcloud.com/server/19/developer_manual/client_apis/LoginFlow/index.html#login-flow-v2).
/// You can use [register_login_flow_2](LoginDetails::register_login_flow_2) to do this authentication
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginDetails {
    pub server: Url,
    #[serde(rename = "loginName")]
    pub login_name: String,
    #[serde(rename = "appPassword")]
    pub app_password: String,
}

impl LoginDetails {
    /// Login with the login flow v2 to the server. The `auth_callback` is given the URL where the
    /// user will grant the permissions, this function should not block (or the authentication will
    /// never finish) waiting for the end of the login_flow.
    pub async fn register_login_flow_2(
        server: Url,
        mut auth_callback: impl FnMut(Url),
    ) -> Result<Self, Error> {
        #[derive(Deserialize)]
        struct Poll {
            token: String,
            endpoint: Url,
        }
        #[derive(Deserialize)]
        struct PollRequest {
            poll: Poll,
            login: Url,
        }
        #[derive(Serialize)]
        struct Token {
            token: String,
        }
        let client = reqwest::Client::new();
        let resp = client
            .post(&format!("{}index.php/login/v2", server))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(Error::LoginFlowError(resp.status().as_u16()));
        }

        let resp: PollRequest = resp.json().await?;
        log::debug!("Got poll request for login_flow_v2");
        auth_callback(resp.login);
        let token = Token {
            token: resp.poll.token,
        };
        let details: LoginDetails = loop {
            let poll = client
                .post(resp.poll.endpoint.as_str())
                .form(&token)
                .send()
                .await?;
            log::debug!("Polled endpoint");
            match poll.status().as_u16() {
                404 => {
                    log::debug!("Not ready, need to retry");
                    tokio::time::delay_for(std::time::Duration::from_millis(100)).await
                }
                200 => break poll.json().await?,
                code => return Err(Error::LoginFlowError(code)),
            }
        };
        Ok(details)
    }
}

/// The state needed to re-connect to a nextcloud instance
#[derive(Serialize, Deserialize, Clone)]
pub struct ResumeState {
    server_url: Url,
    password_url: String,

    keepalive: u64,
    session_id: String,
    shutdown_time: std::time::SystemTime,

    login: String,
    password: String,
}

/// The main entrypoint to the nextcloud API
pub struct AuthenticatedApi {
    server_url: Url,
    client: Client,
    passwords_url: String,

    session_id: String,
    keepalive: u64,

    login: String,
    password: String,
}

impl AuthenticatedApi {
    /// Return the URL of the nextcloud instance
    pub fn server(&self) -> &Url {
        &self.server_url
    }
    async fn reqwest<D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        method: reqwest::Method,
        data: D,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .request(
                method,
                &format!("{}/{}", self.passwords_url, endpoint.as_ref()),
            )
            .json(&data)
            .header("X-API-SESSION", &self.session_id)
            .basic_auth(&self.login, Some(&self.password))
            .send()
            .await
    }
    pub(crate) async fn bytes_request<D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        method: reqwest::Method,
        data: D,
    ) -> Result<bytes::Bytes, Error> {
        let r = self.reqwest(endpoint, method, data).await?;
        r.bytes().await.map_err(Into::into)
    }
    async fn passwords_request<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        method: reqwest::Method,
        data: D,
    ) -> Result<R, Error> {
        let r = self.reqwest(endpoint, method, data).await?;
        let text = r.text().await?;
        let resp = serde_json::from_str(&text).map_err(|e| {
            log::warn!("Response could not be read: {}", text);
            e
        })?;
        match resp {
            EndpointResponse::Success(r) => Ok(r),
            EndpointResponse::Error(e) => Err(Error::EndpointError(e)),
        }
    }
    pub(crate) async fn passwords_get<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        data: D,
    ) -> Result<R, Error> {
        self.passwords_request(endpoint, reqwest::Method::GET, data)
            .await
    }
    pub(crate) async fn passwords_post<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        data: D,
    ) -> Result<R, Error> {
        self.passwords_request(endpoint, reqwest::Method::POST, data)
            .await
    }
    pub(crate) async fn passwords_delete<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        data: D,
    ) -> Result<R, Error> {
        self.passwords_request(endpoint, reqwest::Method::DELETE, data)
            .await
    }
    pub(crate) async fn passwords_patch<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        data: D,
    ) -> Result<R, Error> {
        self.passwords_request(endpoint, reqwest::Method::PATCH, data)
            .await
    }

    /// Access the Password API
    #[inline]
    pub fn password(&self) -> password::PasswordApi<'_> {
        password::PasswordApi { api: self }
    }
    /// Access the Settings API
    #[inline]
    pub fn settings(&self) -> settings::SettingsApi<'_> {
        settings::SettingsApi { api: self }
    }
    /// Access the Folder API
    #[inline]
    pub fn folder(&self) -> folder::FolderApi<'_> {
        folder::FolderApi { api: self }
    }
    /// Access the Share API
    #[inline]
    pub fn share(&self) -> share::ShareApi<'_> {
        share::ShareApi { api: self }
    }
    #[inline]
    pub fn service(&self) -> service::ServiceApi<'_> {
        service::ServiceApi { api: self }
    }

    /// Resume a connection to the API using the state. Also gives the session ID
    pub async fn resume_session(resume_state: ResumeState) -> Result<(Self, String), Error> {
        if resume_state.shutdown_time.elapsed()?.as_secs() > resume_state.keepalive {
            log::debug!("Session was too old, creating new session");
            AuthenticatedApi::new_session(LoginDetails {
                server: resume_state.server_url,
                login_name: resume_state.login,
                app_password: resume_state.password,
            })
            .await
        } else {
            log::debug!("Calling keepalive");
            #[derive(Deserialize)]
            struct Keepalive {
                success: bool,
            }
            let client = Client::new();
            let api = AuthenticatedApi {
                server_url: resume_state.server_url,
                client,
                passwords_url: resume_state.password_url,
                session_id: resume_state.session_id,
                keepalive: resume_state.keepalive,
                login: resume_state.login,
                password: resume_state.password,
            };
            let s: Keepalive = api.passwords_get("1.0/session/keepalive", ()).await?;
            assert!(s.success);
            let session_id = api.session_id.clone();
            Ok((api, session_id))
        }
    }
    /// Create a new session to the API, returns the session ID
    pub async fn new_session(login_details: LoginDetails) -> Result<(Self, String), Error> {
        #[derive(Serialize, Deserialize, Debug)]
        struct OpenSession {
            success: bool,
            keys: Vec<String>,
        }
        let client = Client::new();

        let passwords_url = format!("{}index.php/apps/passwords/api/", login_details.server);
        let session_request = client
            .request(
                reqwest::Method::POST,
                &format!("{}/1.0/session/open", passwords_url),
            )
            .basic_auth(&login_details.login_name, Some(&login_details.app_password))
            .send()
            .await?;
        let session_id: String = session_request
            .headers()
            .get("X-API-SESSION")
            .expect("no api session header")
            .to_str()
            .expect("api session is not ascii")
            .into();
        let session: OpenSession = session_request.json().await?;
        if !session.success {
            Err(Error::ConnectionFailed)?
        }

        let mut api = AuthenticatedApi {
            server_url: login_details.server,
            passwords_url,
            client,
            login: login_details.login_name,
            password: login_details.app_password,
            session_id: session_id.clone(),
            keepalive: 0,
        };
        api.keepalive = api.settings().get().session_lifetime().await?;
        log::debug!("Session keepalive is: {}", api.keepalive);

        Ok((api, session_id))
    }

    /// Disconnect from the session
    pub async fn disconnect(self) -> Result<(), Error> {
        #[derive(Deserialize)]
        struct CloseSession {
            success: bool,
        }
        let s: CloseSession = self.passwords_get("1.0/session/close", ()).await.unwrap();

        if !s.success {
            Err(Error::DisconnectionFailed)
        } else {
            Ok(())
        }
    }

    /// Get the state to be able to resume this session
    pub fn get_state(&self) -> ResumeState {
        ResumeState {
            server_url: self.server_url.clone(),
            password_url: self.passwords_url.clone(),

            keepalive: self.keepalive,
            session_id: self.session_id.clone(),

            login: self.login.clone(),
            password: self.password.clone(),

            shutdown_time: std::time::SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {}
