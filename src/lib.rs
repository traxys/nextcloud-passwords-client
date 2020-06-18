use reqwest::Client;
use serde::{Deserialize, Serialize};
pub use url::Url;

#[derive(Serialize, Deserialize)]
pub struct SessionRequest {}

pub struct Api {
    base_url: Url,
    url_path: String,
    client: Client,
}
impl Api {
    pub fn new(base_url: Url) -> Self {
        Self {
            url_path: format!("{}index.php/apps/passwords/api/", base_url),
            base_url,
            client: Client::new(),
        }
    }
    pub async fn session_request(&self) -> SessionRequest {
        self.client
            .get(&format!("{}/1.0/session/request", self.url_path))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Api;
    use url::Url;

    #[test]
    fn construct_api() {
        let _ = Api::new(Url::parse("https://cloud.net").unwrap());
    }
}
