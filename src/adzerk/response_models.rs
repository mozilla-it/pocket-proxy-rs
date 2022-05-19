use super::defaults;
use crate::{
    endpoints::spocs::{Collection, Shim, Spoc, SpocsList, SpocsResponse},
    errors::ProxyError,
};
use actix_web::{http::Uri, web::Query};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

// AdZerk Output Type
#[derive(Deserialize)]
pub struct DecisionResponse {
    pub decisions: HashMap<String, Option<Vec<Decision>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    ad_id: u32,
    flight_id: u32,
    campaign_id: u32,
    priority_id: Option<u32>,
    click_url: String,
    contents: [Content; 1],
    impression_url: String,
    events: Vec<Event>,
}

#[derive(Deserialize)]
pub struct Content {
    data: Data,
    body: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    ct_title: String,
    ct_url: String,
    ct_domain: String,
    ct_excerpt: String,
    ct_sponsor: Option<String>,
    ct_fullimagepath: String,
    #[serde(rename(deserialize = "ctMin_score"))]
    ct_min_score: Option<String>,
    #[serde(rename(deserialize = "ctItem_score"))]
    ct_item_score: Option<String>,
    #[serde(rename(deserialize = "ctDomain_affinities"))]
    ct_domain_affinities: Option<String>,
    ct_cta: Option<String>,
    ct_collection_title: Option<String>,
    ct_is_video: Option<String>,
    ct_sponsored_by_override: Option<String>,
}

#[derive(Deserialize)]
pub struct Event {
    id: u32,
    url: String,
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
                let spocs: Result<Vec<_>, ProxyError> = decisions
                    .into_iter()
                    .flatten()
                    .map(TryInto::try_into)
                    .collect();
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
            for spoc in spocs.iter_mut().skip(1) {
                spoc.collection_title = None;
            }
            let spoc = &mut spocs[0];
            let collection = Collection {
                title: spoc.collection_title.take().unwrap(),
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
        let mut events_map = EventsMap::new(decision.events)?;
        let spoc = Spoc {
            id: decision.ad_id,
            flight_id: decision.flight_id,
            campaign_id: decision.campaign_id,
            title: custom_data.ct_title,
            url: custom_data.ct_url,
            domain: custom_data.ct_domain,
            excerpt: custom_data.ct_excerpt,
            priority: map_priority(decision.priority_id),
            context: format_context(custom_data.ct_sponsor.as_deref()),
            image_src: get_cdn_image(&custom_data.ct_fullimagepath)?,
            raw_image_src: custom_data.ct_fullimagepath,
            shim: Shim {
                click: tracking_url_to_shim(decision.click_url)?,
                impression: tracking_url_to_shim(decision.impression_url)?,
                delete: events_map.remove(17)?,
                save: events_map.remove(20)?,
            },
            parameter_set: "default",
            caps: &defaults::CAPS,
            domain_affinities: get_domain_affinities(custom_data.ct_domain_affinities),
            personalization_models: get_personalization_models(contents.body)?,
            min_score: get_score(custom_data.ct_min_score, 0.1),
            item_score: get_score(custom_data.ct_item_score, 0.2),
            cta: custom_data.ct_cta,
            collection_title: custom_data.ct_collection_title,
            sponsor: custom_data.ct_sponsor,
            sponsored_by_override: custom_data
                .ct_sponsored_by_override
                .map(clean_sponsored_by_override),
            is_video: get_is_video(custom_data.ct_is_video),
        };
        Ok(spoc)
    }
}

struct EventsMap {
    map: HashMap<u32, String>,
}

impl EventsMap {
    fn new(events: Vec<Event>) -> Result<Self, ProxyError> {
        let map = events
            .into_iter()
            .map(|event| Ok((event.id, tracking_url_to_shim(event.url)?)))
            .collect::<Result<_, ProxyError>>()?;
        Ok(Self { map })
    }

    fn remove(&mut self, event_id: u32) -> Result<String, ProxyError> {
        self.map
            .remove(&event_id)
            .ok_or_else(|| ProxyError::new("invalid event i"))
    }
}

fn get_score(score: Option<String>, default: f64) -> f64 {
    score.and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn map_priority(priority_id: Option<u32>) -> u32 {
    priority_id
        .map(|priority_id| match priority_id {
            147517 => 1,
            180843 => 2,
            147518 => 3,
            160722 => 9,
            147520 => 10,
            _ => defaults::PRIORITY,
        })
        .unwrap_or(defaults::PRIORITY)
}

fn get_cdn_image(full_image_path: &str) -> Result<String, ProxyError> {
    match full_image_path.parse::<Uri>()?.host() {
        Some(domain) if domain.ends_with(".zkcdn.net") || domain == "zkcdn.net" => {
            let mut result = "https://img-getpocket.cdn.mozilla.net/direct?".to_owned();
            form_urlencoded::Serializer::new(&mut result)
                .append_pair("url", full_image_path)
                .append_pair("resize", "w618-h310")
                .finish();
            Ok(result)
        }
        _ => Err(ProxyError::new(format!(
            "Invalid AdZerk image url: {}",
            full_image_path
        ))),
    }
}

fn get_domain_affinities(name: Option<String>) -> &'static HashMap<String, u32> {
    name.and_then(|name| defaults::DOMAIN_AFFINITIES.get(&name))
        .unwrap_or(&defaults::EMPTY_DOMAIN_AFFINITIES)
}

fn get_personalization_models(body: Option<String>) -> Result<HashMap<String, u32>, ProxyError> {
    lazy_static! {
        static ref TRUE_VALUES: [Value; 2] = [Value::from(true), Value::from("true")];
    }
    match body {
        None => Ok(HashMap::new()),
        Some(body) => {
            let map: HashMap<String, Value> = serde_json::from_str(&body)?;
            let result: HashMap<String, u32> = map
                .into_iter()
                .filter_map(|(topic, flag)| match topic.strip_prefix("topic_") {
                    Some(topic) if TRUE_VALUES.contains(&flag) => Some((topic.to_owned(), 1)),
                    _ => None,
                })
                .collect();
            Ok(result)
        }
    }
}

fn clean_sponsored_by_override(mut sponsored_by_override: String) -> String {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(r"(?i)^\s*(blank|empty)\s*$").unwrap();
    }
    if REGEX.is_match(&sponsored_by_override) {
        sponsored_by_override.clear();
    }
    sponsored_by_override
}

fn get_is_video(is_video: Option<String>) -> Option<bool> {
    is_video.and_then(|mut is_video| {
        is_video.make_ascii_lowercase();
        match is_video.trim() {
            "y" | "yes" | "t" | "true" | "on" | "1" => Some(true),
            "n" | "no" | "f" | "false" | "off" | "0" => Some(false),
            _ => None,
        }
    })
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
    use crate::adzerk::response_models::{
        clean_sponsored_by_override, get_cdn_image, get_is_video, tracking_url_to_shim,
    };
    use std::collections::HashMap;

    #[test]
    fn test_deserialize_decisions() {
        let _: Vec<Decision> = serde_json::from_str(include_str!("mock_decision.json")).unwrap();
    }

    #[test]
    fn test_tracking_url_to_shim() {
        let test_string: String = "https://example.local/r?e=123&s=456&j=789".to_string();
        let test_result = tracking_url_to_shim(test_string).unwrap();
        assert_eq!(test_result, "0,123,456")
    }

    #[test]
    fn test_is_video() {
        let test_map: HashMap<Option<String>, Option<bool>> = [
            (Some(String::from("t")), Some(true)),
            (Some(String::from("off")), Some(false)),
            (Some(String::from("1")), Some(true)),
        ]
        .iter()
        .cloned()
        .collect();

        for (key, value) in test_map {
            assert_eq!(get_is_video(key), value);
        }
    }

    #[test]
    fn test_clean_sponsored_by_override() {
        let test_map: HashMap<&str, &str> = [
            ("        blank", ""),
            ("king fisher", "king fisher"),
            ("", ""),
        ]
        .iter()
        .cloned()
        .collect();

        for (key, value) in test_map {
            assert_eq!(
                clean_sponsored_by_override(key.to_string()),
                value.to_string()
            );
        }
    }

    #[test]
    fn test_get_cdn_image() {
        let image_url = "https://img-getpocket.cdn.mozilla.net/direct?";
        let test_map: HashMap<&str, String> = [(
            "https://subdomain.zkcdn.net/foo/bar",
            format!(
                "{}&url=https%3A%2F%2Fsubdomain.zkcdn.net%2Ffoo%2Fbar&resize=w618-h310",
                image_url
            ),
        )]
        .iter()
        .cloned()
        .collect();

        for (key, value) in test_map {
            assert_eq!(get_cdn_image(key).unwrap(), value);
        }
    }
}
