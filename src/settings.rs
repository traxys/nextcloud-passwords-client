use serde::{Serialize, Deserialize};
use url::Url;

pub trait Setting: super::private::Sealed {
    fn name(&self) -> String;
}
pub trait WritableSetting: Setting {}

macro_rules! settings {
    (User = $user:ident($valued_user:ident), Server = $server:ident($valued_server:ident) {
        $(User:$user_variant:ident ($user_type:ty), $user_field:ident => $user_setting:expr),*,
        $(Server:$server_variant:ident ($server_type:ty), $server_field:ident => $server_setting:expr),*,
    }) => {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
        pub enum $user {
            $($user_variant,)*
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub enum $valued_user {
            $($user_variant($user_type),)*
        }

        impl $valued_user {
            pub fn kind(&self) -> $user {
                match self {
                    $(
                        Self::$user_variant(_) => $user::$user_variant,
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

        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
        pub enum $server {
            $($server_variant,)*
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub enum $valued_server {
            $($server_variant($server_type),)*
        }
        impl $valued_server {
            pub fn kind(&self) -> $server {
                match self {
                    $(
                        Self::$server_variant(_) => $server::$server_variant,
                    )*
                }
            }
        }

        impl Setting for $server {
            fn name(&self) -> String {
                match self {
                    $(Self::$server_variant => $server_setting,)*
                }.into()
            }
        }

        #[derive(Debug)]
        pub enum SettingValue {
            $(
                $user_variant($user_type),
            )*
            $(
                $server_variant($server_type),
            )*
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
            pub fn new() -> Self {
                Default::default()
            }
            $(
                pub fn $user_field(mut self, value: $user_type) -> Self {
                    self.$user_field = Some(value);
                    self
                }
            )*
        }
    };
}

settings!{
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
        Server: ManualUrl(Url), manual_url => "server.manual.url",
    }
}
impl WritableSetting for UserSettings {}


impl Setting for ClientSettings {
    fn name(&self) -> String {
        format!("client.{}", self.name)
    }
}

pub struct ClientSettings {
    pub name: String,
}
