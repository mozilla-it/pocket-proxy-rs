use std::collections::HashMap;

use crate::{adzerk::client::AdzerkClient, errors::ClassifyError, utils::RequestClientIp};
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
    pub settings: serde_json::Value,
    pub spocs: Vec<Spoc>,
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
    pub parameter_set: String,
    pub caps: serde_json::Value,
    pub domain_affinities: HashMap<String, u32>,
    pub personalization_models: HashMap<String, u32>,
    pub cta: Option<String>,
    pub collection_title: Option<String>,
    pub sponsor: Option<String>,
    pub sponsored_by_override: Option<String>,
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
) -> Result<HttpResponse, ClassifyError> {
    // validate pocket id is a uuid
    let _: uuid::Uuid = spoc.pocket_id.parse()?;

    if spoc.country.is_none() {
        let location = state.geoip.locate(req.client_ip()?)?;

        spoc.country = location.country.map(|s| s.to_owned());
        spoc.region = location.region.map(|s| s.to_owned());
    }

    let decisions = adzerk_client.get_decisions(spoc.into_inner());

    // let status = adzerk_client.delete_user(&user.pocket_id).await?;
    // Ok(HttpResponse::build(status).json(json!({"status": (status == 200) as i32})))
    todo!()
}
