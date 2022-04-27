use std::time::Duration;

use actix_web::http::StatusCode;
use awc::Client;
use serde::Serialize;

use crate::errors::ClassifyError;

#[derive(Serialize)]
struct UserKey<'a> {
    #[serde(rename(serialize = "userKey"))]
    user_key: &'a str,
}

pub struct AdzerkClient {
    http_client: Client,
    base_url: String,
    network_id: u32,
    adzerk_api_key: String,
}

impl AdzerkClient {
    pub fn new(base_url: String, network_id: u32, adzerk_api_key: String) -> Self {
        let http_client = Client::builder().timeout(Duration::from_secs(30)).finish();
        Self {
            http_client,
            base_url,
            network_id,
            adzerk_api_key,
        }
    }

    pub async fn delete_user(&self, pocket_id: &str) -> Result<StatusCode, ClassifyError> {
        let user_key = UserKey {
            user_key: pocket_id,
        };
        let status = self
            .http_client
            .delete(format!("{}/udb/{}/", self.base_url, self.network_id))
            .insert_header(("X-Adzerk-ApiKey", self.adzerk_api_key.as_str()))
            .query(&user_key)
            .unwrap()
            .send()
            .await?
            .status();
        Ok(status)
    }
}
