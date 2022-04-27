use crate::{adzerk_client::AdzerkClient, errors::ClassifyError};
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use serde_derive::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct User {
    pocket_id: String,
}

pub async fn delete_user(
    user: web::Json<User>,
    adzerk_client: Data<AdzerkClient>,
) -> Result<HttpResponse, ClassifyError> {
    let status = adzerk_client.delete_user(&user.pocket_id).await?;
    Ok(HttpResponse::build(status).json(json!({"status": (status == 200) as i32})))
}

#[cfg(test)]
mod tests {
    use crate::adzerk_client::AdzerkClient;
    use actix_web::{
        test::{self, TestRequest},
        web::{self, Data}, App,
    };
    use serde_json::json;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[actix_rt::test]
    async fn test_delete_user_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let adzerk_api_key = "my-cool-api-key".to_string();
        let network_id = 123;
        let mock_adzerk_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path(format!("/udb/{}/", network_id)))
            .and(header("X-Adzerk-ApiKey", adzerk_api_key.as_str()))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_adzerk_server)
            .await;

        let adzerk_client = AdzerkClient::new(mock_adzerk_server.uri(), network_id, adzerk_api_key);

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
        let value: serde_json::Value = test::call_and_read_body_json(&mut service, request).await;
        assert_eq!(value.get("status").unwrap().as_i64().unwrap(), 1);
        // TODO: assert_eq!(request.status(), 200);

        Ok(())
    }
}
