use std::time::Duration;

use actix_web::http::StatusCode;
use awc::Client;
use serde::{Deserialize, Serialize};

use crate::{
    endpoints::spocs::{self, Spoc},
    errors::ClassifyError,
};

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Placement {
    div_name: String,
    network_id: u32,
    site_id: u32,
    ad_types: Vec<u32>,
    zone_ids: Vec<u32>,
    count: u32,
    event_ids: &'static [u32],
}

impl Placement {
    fn new(network_id: u32) -> Self {
        Self {
            div_name: "spocs".to_owned(),
            network_id,
            site_id: 1070098,
            ad_types: vec![2401, 3617],
            zone_ids: vec![217995],
            count: 10,
            event_ids: &[17, 20],
        }
    }

    fn from_spoc_placement(
        placement: spocs::Placement,
        network_id: u32,
        site: Option<u32>,
    ) -> Self {
        let mut result = Placement::new(network_id);
        if !placement.ad_types.is_empty() {
            result.ad_types = placement.ad_types;
        }
        if !placement.zone_ids.is_empty() {
            result.zone_ids = placement.zone_ids;
        }
        if let Some(site) = site {
            result.site_id = site;
        }
        result.div_name = placement.name;
        result
    }
}

#[derive(Serialize)]
struct User {
    key: String,
}

// Adzerk Input Type
#[derive(Serialize)]
struct DecisionBody {
    placements: Vec<Placement>,
    user: User,
    keywords: Vec<String>,
}

// AdZerk Output Type
#[derive(Deserialize)]
pub struct Decisions {}

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

    pub async fn get_decisions(&self, spoc: Spoc) -> Result<Decisions, ClassifyError> {
        // __add_targeting
        let user = User {
            key: spoc.pocket_id,
        };
        let mut keywords = vec![];

        if let Some(country) = spoc.country {
            keywords.push(country);
            if let Some(region) = spoc.region {
                keywords.push(format!(
                    "{country}-{region}",
                    country = keywords[0],
                    region = region
                ));
            }
        }

        // __add_placements && __add_site
        let placements = if spoc.placements.is_empty() {
            vec![Placement::new(self.network_id)]
        } else {
            spoc.placements
                .into_iter()
                .map(|p| Placement::from_spoc_placement(p, self.network_id, spoc.site))
                .collect()
        };

        let decision_body = DecisionBody {
            placements,
            user,
            keywords,
        };

        let status = self
            .http_client
            .post(format!("{}/api/v2", self.base_url))
            .send_json(&decision_body)
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
