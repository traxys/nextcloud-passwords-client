use reqwest::Client;
use serde::{Deserialize, Serialize};
pub use url::Url;

/// Data types to interract with the folder API
pub mod folder;
/// Data types and builders to interact with the passwords API
pub mod password;
/// Data types, helpers and builders to interact with the settings API
pub mod settings;

mod utils;

mod private {
    pub trait Sealed {}

    impl Sealed for super::settings::UserSettings {}
    impl Sealed for super::settings::ServerSettings {}
    impl Sealed for super::settings::ClientSettings {}
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
}

/// Represent how to first connect to a nextcloud instance
/// The best way to obtain some is using [Login flow
/// v2](https://docs.nextcloud.com/server/19/developer_manual/client_apis/LoginFlow/index.html#login-flow-v2)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginDetails {
    pub server: Url,
    #[serde(rename = "loginName")]
    pub login_name: String,
    #[serde(rename = "appPassword")]
    pub app_password: String,
}

/// The state needed to re-connect to a nextcloud instance
#[derive(Serialize, Deserialize)]
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
    async fn passwords_request<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        method: reqwest::Method,
        data: D,
    ) -> Result<R, reqwest::Error> {
        self.client
            .request(
                method,
                &format!("{}/{}", self.passwords_url, endpoint.as_ref()),
            )
            .json(&data)
            .header("X-API-SESSION", &self.session_id)
            .basic_auth(&self.login, Some(&self.password))
            .send()
            .await?
            .json::<R>()
            .await
    }
    pub(crate) async fn passwords_get<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        data: D,
    ) -> Result<R, reqwest::Error> {
        self.passwords_request(endpoint, reqwest::Method::GET, data)
            .await
    }
    pub(crate) async fn passwords_post<R: serde::de::DeserializeOwned, D: serde::Serialize>(
        &self,
        endpoint: impl AsRef<str>,
        data: D,
    ) -> Result<R, reqwest::Error> {
        self.passwords_request(endpoint, reqwest::Method::POST, data)
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
        api.keepalive = api.settings().get_setting().session_lifetime().await?;
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
