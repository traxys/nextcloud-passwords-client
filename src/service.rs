use crate::{AuthenticatedApi, Error};
use serde::{Deserialize, Serialize};

/// Access the service API
pub struct ServiceApi<'a> {
    pub(crate) api: &'a AuthenticatedApi,
}

impl<'a> ServiceApi<'a> {
    /// Generates a password with the users default settings
    pub async fn generate_password_with_user_settings(&self) -> Result<GenerateResponse, Error> {
        self.api.passwords_get("1.0/service/password", ()).await
    }
    /// The password action generates one password with the given settings.
    ///
    /// Notes
    ///  - Generated passwords are checked for security automatically
    ///  - The maximum value for strength is 4
    pub async fn generate_password(
        &self,
        settings: GeneratePassword,
    ) -> Result<GenerateResponse, Error> {
        self.api
            .passwords_post("1.0/service/password", settings)
            .await
    }

    /// The avatar action returns a png avatar icon for the given user id.
    ///
    /// Notes
    ///  - If the user did not specify an avatar a default image will be generated
    pub async fn avatar(
        &self,
        user: uuid::Uuid,
        MiniatureSize(size): MiniatureSize,
    ) -> Result<bytes::Bytes, Error> {
        self.api
            .bytes_request(
                format!("1.0/service/avatar/{user}/{size}", user = user, size = size),
                reqwest::Method::GET,
                (),
            )
            .await
    }

    /// The favicon action returns a png favicon icon for the given domain.
    ///
    /// Notes
    ///  - If no favicon can be found a default image will be generated
    pub async fn favicon(
        &self,
        domain: String,
        MiniatureSize(size): MiniatureSize,
    ) -> Result<bytes::Bytes, Error> {
        self.api
            .bytes_request(
                format!(
                    "1.0/service/favicon/{domain}/{size}",
                    domain = domain,
                    size = size
                ),
                reqwest::Method::GET,
                (),
            )
            .await
    }
    /// The preview action returns a jpeg preview image for the given domain.
    ///
    /// The default width is 640
    /// The default height is 360...
    /// The default view is Desktop
    ///
    /// Notes
    ///  - If no image can be created a default image will be used
    ///  - This action is known to be slow if the cache is empty
    ///  - The width and height must be a multiple of 10
    ///  - The minimum width is 240 pixels
    ///  - The maximum width is 1280 pixels
    ///  - The minimum height is 240 pixels
    ///  - The maximum height is 1280 pixels
    ///  - If a width and height were specified, the image will be cropped to fill the area
    ///  - The width and height can be 0. In this case, it is up to the api to set the optimal value
    ///  - You can specify a range for width and height by passing SIZE..., SIZE...SIZE or ...SIZE where size is a number.
    ///  The left value will be the minimum and the right value the maximum.
    ///  The api will try to generate an image that fits the given values without cropping
    pub async fn preview(
        &self,
        domain: String,
        view: Option<View>,
        width: Option<String>,
        height: Option<String>,
    ) -> Result<bytes::Bytes, Error> {
        let view = view.unwrap_or(View::Desktop).as_str();
        let width = width.unwrap_or("640".into());
        let height = height.unwrap_or("360...".into());

        self.api
            .bytes_request(
                format!(
                    "1.0/service/preview/{domain}/{view}/{width}/{height}",
                    domain = domain,
                    view = view,
                    width = width,
                    height = height,
                ),
                reqwest::Method::GET,
                (),
            )
            .await
    }
}

#[derive(Debug)]
pub enum View {
    Desktop,
    Mobile,
}
impl View {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
        }
    }
}

/// Represent the possible sizes of an avatar
#[derive(Debug)]
pub struct MiniatureSize(u16);
impl MiniatureSize {
    /// The size must be a multiple of 8, The minimum size is 16 pixels, The maximum size is 256 pixels
    pub fn new(size: u16) -> Option<Self> {
        if size % 8 != 0 || size < 16 || size > 256 {
            None
        } else {
            Some(Self(size))
        }
    }
}
impl Default for MiniatureSize {
    fn default() -> Self {
        MiniatureSize(32)
    }
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct PasswordStrength(u8);
impl PasswordStrength {
    #[inline]
    pub fn one() -> Self {
        PasswordStrength(1)
    }
    #[inline]
    pub fn two() -> Self {
        PasswordStrength(2)
    }
    #[inline]
    pub fn three() -> Self {
        PasswordStrength(3)
    }
    #[inline]
    pub fn four() -> Self {
        PasswordStrength(4)
    }
    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Serialize, Default, Debug)]
pub struct GeneratePassword {
    #[serde(skip_serializing_if = "Option::is_none")]
    strength: Option<PasswordStrength>,
    #[serde(skip_serializing_if = "Option::is_none")]
    numbers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    special: Option<bool>,
}
impl GeneratePassword {
    pub fn new() -> Self {
        Default::default()
    }

    /// A higher value creates a longer and more complex password (Default = 1)
    pub fn strength(self, strength: PasswordStrength) -> Self {
        Self {
            strength: Some(strength),
            ..self
        }
    }
    /// Whether or not numbers should be used in the password (Default = false)
    pub fn numbers(self, numbers: bool) -> Self {
        Self {
            numbers: Some(numbers),
            ..self
        }
    }
    /// Whether or not special characters should be used in the password
    pub fn special(self, special: bool) -> Self {
        Self {
            special: Some(special),
            ..self
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateResponse {
    pub password: String,
    pub words: String,
    pub strength: u8,
    pub numbers: bool,
    pub special: bool,
}
