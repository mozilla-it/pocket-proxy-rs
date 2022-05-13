use super::request_models::Placement;
use lazy_static::lazy_static;
use serde_json::{from_str, json, Value};
use std::collections::HashMap;

pub const NETWORK_ID: u32 = 10250;

lazy_static! {
    pub static ref BASE_URL: String = format!("https://e-{0}.adzerk.net", NETWORK_ID);
    pub static ref PLACEMENT: Placement = Placement {
        div_name: "spocs".to_owned(),
        network_id: NETWORK_ID,
        site_id: 1070098,
        ad_types: vec![2401, 3617],
        zone_ids: vec![217995],
        count: 10,
        event_ids: [17, 20],
    };
    pub static ref CAPS: Value = json!({
        "lifetime": 50,
        "campaign": {
            "count": 10,
            "period": 86400,
        },
        "flight": {
            "count": 10,
            "period": 86400,
        },
    });
    pub static ref SETTINGS: Value = from_str(include_str!("settings.json")).unwrap();
    pub static ref DOMAIN_AFFINITIES: HashMap<String, HashMap<String, u32>> =
        from_str(include_str!("domain_affinities.json")).unwrap();
}

#[cfg(test)]
mod tests {
    use super::{DOMAIN_AFFINITIES, SETTINGS};
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_parse_json_files() {
        let _: &Value = &SETTINGS;
        let _: &HashMap<_, _> = &DOMAIN_AFFINITIES;
    }
}
