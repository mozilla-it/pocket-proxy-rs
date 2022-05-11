use crate::endpoints::spocs::{self, SpocsRequest};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserKey<'a> {
    #[serde(rename(serialize = "userKey"))]
    pub user_key: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub div_name: String,
    pub network_id: u32,
    pub site_id: u32,
    pub ad_types: Vec<u32>,
    pub zone_ids: Vec<u32>,
    pub count: u32,
    pub event_ids: [u32; 2],
}

impl Placement {
    pub fn new(network_id: u32) -> Self {
        Self {
            div_name: "spocs".to_owned(),
            network_id,
            site_id: 1070098,
            ad_types: vec![2401, 3617],
            zone_ids: vec![217995],
            count: 10,
            event_ids: [17, 20],
        }
    }

    pub fn from_spoc_placement(
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
pub struct User {
    pub key: String,
}

// Adzerk Input Type
#[derive(Serialize)]
pub struct DecisionRequest {
    pub placements: Vec<Placement>,
    pub user: User,
    pub keywords: Vec<String>,
}

impl DecisionRequest {
    pub fn from_spocs_request(spoc: SpocsRequest, network_id: u32) -> Self {
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
            vec![Placement::new(network_id)]
        } else {
            spoc.placements
                .into_iter()
                .map(|p| Placement::from_spoc_placement(p, network_id, spoc.site))
                .collect()
        };

        DecisionRequest {
            placements,
            user,
            keywords,
        }
    }
}

// AdZerk Output Type
#[derive(Deserialize)]
pub struct DecisionResponse {}

#[cfg(test)]
mod tests {
    use super::DecisionRequest;
    use crate::endpoints::spocs::SpocsRequest;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn test_request_conversion() {
        let spoc_request: SpocsRequest = from_value(json!({
            "consumer_key": "40249-e88c401e1b1f2242d9e441c4",
            "placements": [
                {
                    "ad_types": [
                        3617
                    ],
                    "name": "spocs",
                    "zone_ids": [
                        217758,
                        217995
                    ]
                }
            ],
            "pocket_id": "{670e8b97-c271-483f-bcb0-4921b58cdb52}",
            "version": 2,
            "country": "US",
            "region": "IL"
        }))
        .unwrap();
        let expected_decision_request = json!({
            "placements": [
                {
                    "adTypes": [
                        3617
                    ],
                    "count": 10,
                    "divName": "spocs",
                    "eventIds": [
                        17,
                        20
                    ],
                    "networkId": 10250,
                    "siteId": 1070098,
                    "zoneIds": [
                        217758,
                        217995
                    ]
                }
            ],
            "user": {
                "key": "{670e8b97-c271-483f-bcb0-4921b58cdb52}"
            },
            "keywords": ["US", "US-IL"]
        });
        let actual_decision_request =
            to_value(DecisionRequest::from_spocs_request(spoc_request, 10250)).unwrap();
        assert_eq!(actual_decision_request, expected_decision_request);
    }
}
