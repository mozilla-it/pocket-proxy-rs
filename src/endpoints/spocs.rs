use std::collections::HashMap;

use crate::{adzerk::client::AdzerkClient, errors::ProxyError, utils::RequestClientIp};
use actix_web::{
    web::{self, Data},
    HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};

use super::EndpointState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpocsRequest {
    pub version: u32,
    pub consumer_key: String,
    pub pocket_id: String,
    pub site: Option<u32>,
    #[serde(default)]
    pub placements: Vec<Placement>,
    pub country: Option<String>,
    pub region: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub name: String,
    #[serde(default)]
    pub zone_ids: Vec<u32>,
    #[serde(default)]
    pub ad_types: Vec<u32>,
    pub count: Option<u32>,
}

#[derive(Serialize)]
pub struct SpocsResponse {
    pub settings: &'static serde_json::Value,
    #[serde(flatten)]
    pub divs: HashMap<String, SpocsList>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SpocsList {
    Standard(Vec<Spoc>),
    Collection(Collection),
}

#[derive(Serialize)]
pub struct Collection {
    pub title: String,
    pub flight_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    pub context: String,
    pub items: Vec<Spoc>,
}

#[derive(Serialize)]
pub struct Spoc {
    pub id: u32,
    pub flight_id: u32,
    pub campaign_id: u32,
    pub title: String,
    pub url: String,
    pub domain: String,
    pub excerpt: String,
    pub priority: u32,
    pub context: String,
    pub raw_image_src: String,
    pub image_src: String,
    pub shim: Shim,
    pub parameter_set: &'static str,
    pub caps: &'static serde_json::Value,
    pub domain_affinities: &'static HashMap<String, u32>,
    pub personalization_models: HashMap<String, u32>,
    pub min_score: f64,
    pub item_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsored_by_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_video: Option<bool>,
}

#[derive(Serialize)]
pub struct Shim {
    pub click: String,
    pub impression: String,
    pub delete: String,
    pub save: String,
}

pub async fn spocs(
    mut spoc: web::Json<SpocsRequest>,
    state: Data<EndpointState>,
    adzerk_client: Data<AdzerkClient>,
    req: HttpRequest,
) -> Result<HttpResponse, ProxyError> {
    // validate pocket id is a uuid
    let _: uuid::Uuid = spoc.pocket_id.parse()?;

    if spoc.country.is_none() {
        if let Ok(location) = state.geoip.locate(req.client_ip()?) {
            spoc.country = location.country.map(|s| s.to_owned());
            spoc.region = location.region.map(|s| s.to_owned());
        }
    }

    let spocs_response = adzerk_client.get_decisions(spoc.into_inner()).await?;

    Ok(HttpResponse::Ok().json(spocs_response))
}
