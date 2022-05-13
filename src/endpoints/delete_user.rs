use crate::{adzerk::client::AdzerkClient, errors::ProxyError};
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct User {
    pocket_id: String,
}

#[derive(Serialize)]
pub struct DeleteUserResponse {
    status: u32,
}

pub async fn delete_user(
    user: web::Json<User>,
    adzerk_client: Data<AdzerkClient>,
) -> Result<HttpResponse, ProxyError> {
    let status = adzerk_client.delete_user(&user.pocket_id).await?;
    let response_body = DeleteUserResponse {
        status: (status == 200) as _,
    };
    Ok(HttpResponse::build(status).json(response_body))
}

#[cfg(test)]
mod tests {
    use crate::adzerk::{client::AdzerkClient, defaults};
    use actix_web::{
        test::{self, TestRequest},
        web::{self, Data},
        App,
    };
    use serde_json::{json, Value};
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[actix_rt::test]
    async fn test_delete_user_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let adzerk_api_key = "my-cool-api-key";
        let mock_adzerk_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path(format!("/udb/{}/", defaults::NETWORK_ID)))
            .and(header("X-Adzerk-ApiKey", adzerk_api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_adzerk_server)
            .await;

        let adzerk_client =
            AdzerkClient::new(adzerk_api_key.into()).with_base_url(mock_adzerk_server.uri());

        let mut service = test::init_service(
            App::new()
                .app_data(Data::new(adzerk_client))
                .route("/user", web::delete().to(super::delete_user)),
        )
        .await;

        let request = TestRequest::delete()
            .uri("/user")
            .set_json(json!({"pocket_id": "{123}"}))
            .to_request();
        let response: Value = test::call_and_read_body_json(&mut service, request).await;
        assert_eq!(response, json!({"status": 1}));
        // TODO: assert_eq!(request.status(), 200);

        Ok(())
    }
}
