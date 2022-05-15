use super::defaults;
use crate::{
    endpoints::spocs::{Collection, Spoc, SpocsList, SpocsResponse},
    errors::ProxyError,
};
use actix_web::{http::Uri, web::Query};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

// AdZerk Output Type
#[derive(Deserialize)]
pub struct DecisionResponse {
    pub decisions: HashMap<String, Vec<Decision>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    pub ad_id: u32,
    pub flight_id: u32,
    pub campaign_id: u32,
    pub priority_id: Option<u32>,
    pub click_url: String,
    pub contents: [Content; 1],
    pub impression_url: String,
    pub events: Vec<Event>,
}

#[derive(Deserialize)]
pub struct Content {
    #[serde(rename(deserialize = "type"))]
    pub type_: String,
    pub data: Data,
    pub body: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub ct_title: String,
    pub ct_url: String,
    pub ct_domain: String,
    pub ct_excerpt: String,
    pub ct_sponsor: Option<String>,
    pub ct_fullimagepath: String,
    #[serde(rename(deserialize = "ctMin_score"))]
    pub ct_min_score: Option<f64>,
    #[serde(rename(deserialize = "ctItem_score"))]
    pub ct_item_score: Option<f64>,
    #[serde(rename(deserialize = "ctDomain_affinities"))]
    pub ct_domain_affinities: String,
    pub ct_collection_title: Option<String>,
    pub ct_is_video: Option<String>,
    pub ct_image: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Deserialize)]
pub struct Event {
    pub id: u32,
    pub url: String,
}

impl SpocsResponse {
    pub fn from_decision_response(
        decision_response: DecisionResponse,
        version: u32,
    ) -> Result<Self, ProxyError> {
        let divs = decision_response
            .decisions
            .into_iter()
            .map(|(div, decisions)| {
                let spocs: Result<Vec<_>, ProxyError> =
                    decisions.into_iter().map(TryInto::try_into).collect();
                let spoc_list = SpocsList::from_spocs(spocs?, version);
                Ok((div, spoc_list))
            })
            .collect::<Result<_, ProxyError>>()?;
        Ok(SpocsResponse {
            settings: &defaults::SETTINGS,
            divs,
        })
    }
}

impl SpocsList {
    fn from_spocs(mut spocs: Vec<Spoc>, version: u32) -> Self {
        if version >= 2 && !spocs.is_empty() && spocs.iter().all(|s| s.collection_title.is_some()) {
            for spoc in spocs.iter_mut() {
                spoc.collection_title = None;
            }
            let spoc = &spocs[0];
            let collection = Collection {
                title: spoc.collection_title.clone().unwrap(),
                flight_id: spoc.flight_id,
                sponsor: spoc.sponsor.clone(),
                context: format_context(spoc.sponsor.as_deref()),
                items: spocs,
            };
            SpocsList::Collection(collection)
        } else {
            SpocsList::Standard(spocs)
        }
    }
}

fn format_context(sponsor: Option<&str>) -> String {
    sponsor
        .map(|s| format!("Sponsored by {}", s))
        .unwrap_or_default()
}

impl TryFrom<Decision> for Spoc {
    type Error = ProxyError;

    fn try_from(decision: Decision) -> Result<Self, Self::Error> {
        let [contents] = decision.contents;
        let custom_data = contents.data;
        let body: Option<HashMap<String, Value>> = contents
            .body
            .map(|body| serde_json::from_str(&body))
            .transpose()?;
        let event_map: HashMap<u32, String> = decision
            .events
            .into_iter()
            .map(|event| Ok((event.id, tracking_url_to_shim(event.url)?)))
            .collect::<Result<_, ProxyError>>()?;
        todo!()
    }
}

#[derive(Deserialize)]
struct TrackingParameters {
    e: String,
    s: String,
}

fn tracking_url_to_shim(url: String) -> Result<String, ProxyError> {
    let url: Uri = url.parse()?;
    let path_id = match url.path() {
        "/r" => '0',
        "/i.gif" => '1',
        "/e.gif" => '2',
        _ => {
            return Err(ProxyError::new(format!(
                "Unknown telemetry path: '{}'",
                url.path()
            )))
        }
    };
    let params =
        Query::<TrackingParameters>::from_query(url.query().unwrap_or_default())?.into_inner();
    Ok(format!("{},{},{}", path_id, params.e, params.s))
}

#[cfg(test)]
mod tests {
    use super::Decision;

    #[test]
    fn test_deserialize_responses() {
        let _: Vec<Decision> = serde_json::from_str(include_str!("mock_decision.json")).unwrap();
    }
}
