use crate::AuthenticatedApi;
use serde::{Deserialize, Serialize};
use url::Url;

/// Fetch a single setting
pub struct SettingsFetcher<'api> {
    pub(crate) api: &'api AuthenticatedApi,
}
pub struct SettingReset<'api> {
    pub(crate) api: &'api AuthenticatedApi,
}

/// Represent a way to map the settings enums to nextcloud passwords
/// setting names
pub trait Setting: super::private::Sealed {
    fn name(&self) -> String;
}
/// What settings are writable by the API
pub trait WritableSetting: Setting {}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("could not parse a number")]
    Number(#[from] std::num::ParseIntError),

    #[error("could not parse a boolean")]
    Boolean(#[from] std::str::ParseBoolError),
}

macro_rules! settings {
    (@dollar[$dol:tt] User = $user:ident($valued_user:ident), Server = $server:ident($valued_server:ident) {
        $(User:$user_variant:ident ($user_type:ty), $user_field:ident => $user_setting:expr),*,
        $(Server:$server_variant:ident ($server_type:ty), $server_field:ident => $server_setting:expr),*,
    }) => {

        /// The names of all the settings
        pub const SETTINGS_NAMES: &'static [&'static str] = &[
            $($user_setting,)*
            $($server_setting,)*
        ];
        /// The names of user settings
        pub const USER_SETTING_NAMES: &'static [&'static str] = &[
            $($user_setting,)*
        ];
        /// The name of server settings
        pub const SERVER_SETTING_NAMES: &'static [&'static str] = &[
            $($server_setting,)*
        ];

        /// takes a macro with the signature `callback!($variant_name:ident; $type:ty; $field_name:ident; $setting_string:expr => $(arg:tt)*)`
        /// and expands it for each of the settings (client not included).
        ///
        /// You need to prefix by expr if an expr is generated, and item if an item is generated
        /// example call: `macro_on_settings!(callback(...args))`. Notice that there is no `!`
        #[macro_export]
        macro_rules! macro_on_settings {
            ($dol callback:ident ($dol ($dol args:tt)*)) => {
                $(
                    $dol callback!($user_variant; $user_type; $user_field; $user_setting => $dol ($dol args)*);
                )*
                $(
                    $dol callback!($server_variant; $server_type; $server_field; $server_setting => $dol ($dol args)*);
                )*
            };
        }
        /// Check the definition of [macro_on_settings]
        #[macro_export]
        macro_rules! macro_on_user_settings {
            ($dol callback:ident ($dol ($dol args:tt)*)) => {
                $(
                    $dol callback!($user_variant; $user_type; $user_field; $user_setting => $dol ($dol args)*);
                )*
            };
        }
        /// Check the definition of [macro_on_settings]
        #[macro_export]
        macro_rules! macro_on_server_settings {
            ($dol callback:ident ($dol ($dol args:tt)*)) => {
                $(
                    $dol callback!($user_variant; $user_type; $user_field; $user_setting => $dol ($dol args)*);
                )*
            };
        }

        /// User Setting names
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
        pub enum $user {
            $($user_variant,)*
        }

        impl std::str::FromStr for $user {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $user_setting => Ok(Self::$user_variant),
                    )*
                        _ => Err("Unknown variant"),
                }
            }
        }

        impl<'api> SettingReset<'api> {
            $(
                pub async fn $user_field(&self) -> Result<$user_type, crate::Error> {
                    let data: Settings = self.api.passwords_post("1.0/settings/reset", vec![$user_setting]).await?;
                    Ok(data.$user_field.expect("server did not provide the asked setting"))
                }
            )*

        }

        impl<'api> SettingsFetcher<'api> {
            $(
                pub async fn $user_field(&self) -> Result<$user_type, crate::Error> {
                    let data: Settings = self.api.passwords_post("1.0/settings/get", vec![$user_setting]).await?;
                    Ok(data.$user_field.expect("server did not provide the asked setting"))
                }
            )*
            $(
                pub async fn $server_field(&self) -> Result<$server_type, crate::Error> {
                    let data: Settings = self.api.passwords_post("1.0/settings/get", vec![$server_setting]).await?;
                    Ok(data.$server_field.expect("server did not provide the asked setting"))
                }
            )*
            pub async fn client_setting<D: serde::de::DeserializeOwned>(&self, client_setting: ClientSettings) -> Result<Option<D>, crate::Error> {
                let mut data: std::collections::HashMap<String, Option<D>> = self.api.passwords_post("1.0/settings/get", vec![client_setting.name()]).await?;
                Ok(data.remove(&client_setting.name()).flatten())
            }
            pub async fn from_variant(&self, variant: SettingVariant) -> Result<SettingValue, crate::Error> {
                match variant {
                    SettingVariant::Client => Err(crate::Error::InvalidSetting),
                    variant => {
                        let data: Settings = self.api.passwords_post("1.0/settings/get", vec![variant.name()]).await?;
                        Ok(data.to_values().pop().unwrap())
                    }
                }
            }
        }


        /// User Setting Values
        #[derive(Serialize, Deserialize, Debug)]
        pub enum $valued_user {
            $($user_variant($user_type),)*
        }

        impl $valued_user {
            /// Return the name of the setting
            pub fn kind(&self) -> $user {
                match self {
                    $(
                        Self::$user_variant(_) => $user::$user_variant,
                    )*
                }
            }
            $(
                /// Coerce the value of the setting to this setting
                pub fn $user_field(self) -> Result<$user_type, Self> {
                    match self {
                        Self::$user_variant(v) => Ok(v),
                        _ => Err(self),
                    }
                }
            )*
            pub fn from_variant(variant: $user, value: &str) -> Result<Self, ParseError> {
                match variant {
                    $(
                        $user::$user_variant => Ok($valued_user::$user_variant(value.parse()?)),
                    )*
                }
            }
        }

        impl Setting for $user {
            fn name(&self) -> String {
                match self {
                    $(Self::$user_variant => $user_setting,)*
                }.into()
            }
        }

        /// Server Setting Names
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
        pub enum $server {
            $($server_variant,)*
        }

        impl std::str::FromStr for $server {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $server_setting => Ok(Self::$server_variant),
                    )*
                        _ => Err("Unknown variant"),
                }
            }
        }

        /// Server Setting Values
        #[derive(Serialize, Deserialize, Debug)]
        pub enum $valued_server {
            $($server_variant($server_type),)*
        }
        impl $valued_server {
            /// Return the name of the setting
            pub fn kind(&self) -> $server {
                match self {
                    $(
                        Self::$server_variant(_) => $server::$server_variant,
                    )*
                }
            }
            $(
                /// Coerce the value of the setting to this setting
                pub fn $server_field(self) -> Result<$server_type, Self> {
                    match self {
                        Self::$server_variant(v) => Ok(v),
                        _ => Err(self),
                    }
                }
            )*
        }

        impl Setting for $server {
            fn name(&self) -> String {
                match self {
                    $(Self::$server_variant => $server_setting,)*
                }.into()
            }
        }

        /// The value of a Setting
        #[derive(Serialize, Deserialize, Debug)]
        pub enum SettingValue {
            $(
                $user_variant($user_type),
            )*
            $(
                $server_variant($server_type),
            )*
            Client { name: String, value: String }
        }
        /// Setting name
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
        pub enum SettingVariant {
            $(
                $user_variant,
            )*
            $(
                $server_variant,
            )*
            Client,
        }
        impl SettingVariant {
            pub(crate) fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$user_variant => $user_setting,
                    )*
                    $(
                        Self::$server_variant => $server_setting,
                    )*
                    Self::Client => panic!("client has no name"),
                }
            }
        }
        impl std::str::FromStr for SettingVariant {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $user_setting => Ok(Self::$user_variant),
                    )*
                    $(
                        $server_setting => Ok(Self::$server_variant),
                    )*
                        _ => Err("Unknown variant"),
                }
            }
        }

        impl From<$user> for SettingVariant {
            fn from(value: $user) -> Self {
                match value {
                    $(
                        $user::$user_variant => SettingVariant::$user_variant,
                    )*
                }
            }
        }
        impl From<$server> for SettingVariant {
            fn from(value: $server) -> Self {
                match value {
                    $(
                        $server::$server_variant => SettingVariant::$server_variant,
                    )*
                }
            }
        }

        impl From<$valued_user> for SettingValue {
            fn from(value: $valued_user) -> Self {
                match value {
                    $(
                        $valued_user::$user_variant(v) => SettingValue::$user_variant(v),
                    )*
                }
            }
        }
        impl From<$valued_server> for SettingValue {
            fn from(value: $valued_server) -> Self {
                match value {
                    $(
                        $valued_server::$server_variant(v) => SettingValue::$server_variant(v),
                    )*
                }
            }
        }

        #[derive(Serialize, Deserialize, Default, Debug)]
        /// Set the values of settings
        pub struct Settings {
            $(
                #[serde(skip_serializing_if = "Option::is_none", rename = $user_setting)]
                $user_field: Option<$user_type>,
            )*
            $(
                #[serde(skip_serializing_if = "Option::is_none", rename = $server_setting)]
                $server_field: Option<$server_type>,
            )*
        }

        #[derive(Serialize, Deserialize, Debug)]
        /// The value of all settings
        pub struct AllSettings {
            $(
                #[serde(rename = $user_setting)]
                $user_field: $user_type,
            )*
            $(
                #[serde(rename = $server_setting)]
                $server_field: $server_type,
            )*
        }

        impl Settings {
            pub(crate) fn to_values(self) -> Vec<SettingValue> {
                let mut settings = Vec::new();
                $(
                    if let Some(value) = self.$user_field {
                        settings.push($valued_user::$user_variant(value).into())
                    }
                )*
                $(
                    if let Some(value) = self.$server_field {
                        settings.push($valued_server::$server_variant(value).into())
                    }
                )*
                settings
            }
            /// Empty settings
            pub fn new() -> Self {
                Default::default()
            }
            $(
                /// Assign a value to this setting
                pub fn $user_field(mut self, value: $user_type) -> Self {
                    self.$user_field = Some(value);
                    self
                }
            )*
            pub fn set_user_value(mut self, setting: $valued_user) -> Self {
                match setting {
                    $(
                        $valued_user::$user_variant(v) => self.$user_field = Some(v),
                    )*
                }
                self
            }
        }
    };

    ( $($input:tt)*) => {
        settings!{
            @dollar[$] $($input)*
        }
    };
}

settings! {
    User = UserSettings(UserSettingValue), Server = ServerSettings(ServerSettingValue) {
        User: PasswordStrength(i8), password_strength => "user.password.generator.strength",
        User: PasswordContainsNumber(bool), password_contains_numbers => "user.password.generator.numbers",
        User: PasswordContainsSpecial(bool), password_contains_special => "user.password.generator.special",
        User: CheckForDuplicates(bool), check_for_duplicates => "user.password.security.duplicates",
        User: CheckForOldPasswords(i64), check_for_old_passwords => "user.password.security.age",
        User: NotifySecurityByMail(bool), notify_security_by_mail => "user.mail.security",
        User: NotifySharesByMail(bool), notify_shares_by_mail => "user.mail.shares",
        User: NotifySecurityByNotification(bool), notify_security_by_notification => "user.notification.security",
        User: NotifySharesByNotification(bool), notify_shares_by_notification => "user.notification.shares",
        User: NotifyErrorsByNotification(bool), notifiy_errors_by_notification => "user.notification.errors",
        User: ServerSideEncryption(i8), server_side_encryption => "user.encryption.sse",
        User: ClientSideEncryption(i8), client_side_encryption => "user.encryption.cse",
        User: SessionLifetime(u64), session_lifetime => "user.session.lifetime",
        Server: Version(String), version => "server.version",
        Server: BaseUrl(Url), base_url => "server.baseUrl",
        Server: BaseUrlWebDav(Url), base_url_web_dav => "server.baseUrl.webdav",
        Server: Sharing(bool), sharing => "server.sharing.enabled",
        Server: Resharing(bool), resharing => "server.sharing.resharing",
        Server: AutoComplete(bool), autocomplete => "server.sharing.autocomplete",
        Server: SharingTypes(Vec<String>), sharing_types => "server.sharing.types",
        Server: PrimaryColor(String), primary_color => "server.theme.color.primary",
        Server: TextColor(String), text_color => "server.theme.color.text",
        Server: BackgroundColor(String), background_color => "server.theme.color.background",
        Server: BackgroundTheme(Url), background_theme => "server.theme.background",
        Server: Logo(Url), logo => "server.theme.logo",
        Server: Label(String), label => "server.theme.label",
        Server: AppIcon(Url), app_icon => "server.theme.app.icon",
        Server: FolderIcon(Url), folder_icon => "server.theme.folder.icon",
        //Server: ManualUrl(Url), manual_url => "server.manual.url",
    }
}
impl WritableSetting for UserSettings {}

impl Setting for ClientSettings {
    fn name(&self) -> String {
        format!("client.{}", self.name)
    }
}

/// An arbitrary client setting
pub struct ClientSettings {
    pub name: String,
}
