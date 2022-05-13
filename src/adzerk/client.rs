use std::time::Duration;

use actix_web::http::StatusCode;
use awc::Client;

use crate::{
    endpoints::spocs::{SpocsRequest, SpocsResponse},
    errors::ProxyError,
};

use super::{
    defaults,
    request_models::{DecisionRequest, UserKey},
};

pub struct AdzerkClient {
    http_client: Client,
    base_url: String,
    adzerk_api_key: String,
}

impl AdzerkClient {
    pub fn new(adzerk_api_key: String) -> Self {
        let http_client = Client::builder().timeout(Duration::from_secs(30)).finish();
        let base_url = defaults::BASE_URL.clone();
        Self {
            http_client,
            base_url,
            adzerk_api_key,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub async fn delete_user(&self, pocket_id: &str) -> Result<StatusCode, ProxyError> {
        let user_key = UserKey {
            user_key: pocket_id,
        };
        let status = self
            .http_client
            .delete(format!("{}/udb/{}/", self.base_url, defaults::NETWORK_ID))
            .insert_header(("X-Adzerk-ApiKey", self.adzerk_api_key.as_str()))
            .query(&user_key)
            .unwrap()
            .send()
            .await?
            .status();
        Ok(status)
    }

    pub async fn get_decisions(&self, spoc: SpocsRequest) -> Result<SpocsResponse, ProxyError> {
        let decision_request = DecisionRequest::from(spoc);
        let status = self
            .http_client
            .post(format!("{}/api/v2", self.base_url))
            .send_json(&decision_request)
            .await?;

        // TODO: Return something to the client
        // if status is "bad"
        if status.status().as_u16() == 400 {
            // idk
        }

        // TODO: transform.py decisions

        todo!()
    }
}
