use crate::{adzerk_client::AdzerkClient, errors::ClassifyError};
use actix_web::{web::{self, Data}, HttpResponse};
use serde::Deserialize;

use super::EndpointState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spoc {
    version: String,
    consumer_key: String,
    pocket_id: String,
    site: Option<String>,
    #[serde(default)]
    placements: Vec<Placement>,
    country: Option<String>,
    region: Option<String>,
}

#[derive(Deserialize)]
pub struct Placement {
    name: String,
    #[serde(default)]
    zone_ids: Vec<u32>,
    #[serde(default)]
    ad_types: Vec<u32>,
    count: Option<u32>,
}

pub async fn spocs(
    spoc: web::Json<Spoc>,
    state: Data<EndpointState>,
    adzerk_client: Data<AdzerkClient>,
) -> Result<HttpResponse, ClassifyError> {
    // validate pocket id is a uuid
    let _: uuid::Uuid = spoc.pocket_id.parse()?;

    // let status = adzerk_client.delete_user(&user.pocket_id).await?;
    // Ok(HttpResponse::build(status).json(json!({"status": (status == 200) as i32})))
    todo!()
}
