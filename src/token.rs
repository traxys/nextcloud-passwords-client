use crate::{AuthenticatedApi, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Interact with the Token API
pub struct TokenApi<'a> {
    pub(crate) api: &'a AuthenticatedApi,
}

impl<'a> TokenApi<'a> {
    /// The request action is required for some token in order to send the user the token.
    /// For example the email token will send an email to the users mail account.
    /// It is recommended to only call this action if the user has chosen that token, not just trigger it for all available tokens.
    pub async fn request(&self, provider: &str) -> Result<Response, Error> {
        self.api
            .passwords_get(
                format!("1.0/token/{provider}/request", provider = provider),
                (),
            )
            .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub data: Value,
}
