use crate::errors::ProxyError;

use actix_web::{http::StatusCode, HttpResponse};
use serde_derive::Serialize;

#[derive(Serialize)]
struct PulseResponse {
    pulse: String,
}

pub async fn pulse() -> Result<HttpResponse, ProxyError> {
    let response = PulseResponse {
        pulse: "ok".to_string(),
    };

    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[actix_web::test]
    async fn pulse_ok() {
        let resp = pulse().await;
        assert_eq!(resp.unwrap().status(), StatusCode::OK);
    }
}
