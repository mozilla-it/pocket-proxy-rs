use actix_web::HttpResponse;
use serde_derive::Serialize;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyError {
    message: String,
}

impl ProxyError {
    pub fn new<M: Into<String>>(message: M) -> Self {
        let message = message.into();
        Self { message }
    }

    pub fn from_source<S: fmt::Display, E: fmt::Display>(source: S, err: E) -> Self {
        Self {
            message: format!("{}: {}", source, err),
        }
    }
}

// Use default implementation of Error
impl std::error::Error for ProxyError {}

impl fmt::Display for ProxyError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)?;
        Ok(())
    }
}

impl actix_web::error::ResponseError for ProxyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(self)
    }
}

macro_rules! impl_from_error {
    ($error: ty) => {
        impl From<$error> for ProxyError {
            fn from(error: $error) -> Self {
                Self::from_source(stringify!($error), error)
            }
        }
    };
}

impl_from_error!(actix_web::http::header::ToStrError);
impl_from_error!(envy::Error);
impl_from_error!(ipnet::AddrParseError);
impl_from_error!(maxminddb::MaxMindDBError);
impl_from_error!(std::io::Error);
impl_from_error!(std::net::AddrParseError);
impl_from_error!(awc::error::SendRequestError);
impl_from_error!(awc::error::PayloadError);
impl_from_error!(awc::error::JsonPayloadError);
impl_from_error!(uuid::Error);
impl_from_error!(serde_json::Error);
impl_from_error!(actix_web::http::uri::InvalidUri);
impl_from_error!(actix_web::error::QueryPayloadError);
