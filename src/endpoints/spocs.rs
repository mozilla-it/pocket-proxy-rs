use crate::{adzerk_client::AdzerkClient, errors::ClassifyError, utils::RequestClientIp};
use actix_web::{
    web::{self, Data},
    HttpRequest, HttpResponse,
};
use serde::Deserialize;

use super::EndpointState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spoc {
    pub version: String,
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

pub async fn spocs(
    mut spoc: web::Json<Spoc>,
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
