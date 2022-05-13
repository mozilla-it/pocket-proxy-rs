//! A server that tells clients what time it is and where they are in the world.
//!
#![deny(clippy::all)]

pub mod adzerk;
pub mod endpoints;
pub mod errors;
pub mod geoip;
pub mod logging;
pub mod metrics;
pub mod settings;
pub mod utils;

use crate::{
    adzerk::client::AdzerkClient,
    endpoints::{debug, delete_user, dockerflow, EndpointState},
    errors::ProxyError,
    geoip::GeoIp,
    settings::Settings,
};
use actix_web::{
    web::{self, Data},
    App,
};

use std::sync::Arc;

const APP_NAME: &str = "pocket-proxy";

#[actix_web::main]
async fn main() -> Result<(), ProxyError> {
    let Settings {
        debug,
        geoip_db_path,
        host,
        human_logs,
        metrics_target,
        port,
        trusted_proxy_list,
        version_file,
        adzerk_api_key,
        ..
    } = Settings::load()?;

    let app_log = logging::get_logger("app", human_logs);

    let metrics = Arc::new(
        metrics::get_client(metrics_target, app_log.clone())
            .unwrap_or_else(|err| panic!("Critical failure setting up metrics logging: {}", err)),
    );

    let state = EndpointState {
        geoip: Arc::new(
            GeoIp::builder()
                .path(geoip_db_path)
                .metrics(Arc::clone(&metrics))
                .build()?,
        ),
        metrics,
        trusted_proxies: trusted_proxy_list,
        log: app_log.clone(),
        version_file,
    };

    let addr = format!("{}:{}", host, port);
    slog::info!(app_log, "starting server on https://{}", addr);

    actix_web::HttpServer::new(move || {
        let adzerk_client = AdzerkClient::new(adzerk_api_key.clone());
        let mut app = App::new()
            .app_data(Data::new(state.clone()))
            .app_data(Data::new(adzerk_client))
            .wrap(metrics::ResponseTimer)
            .wrap(logging::RequestLogger)
            // API Endpoints
            .service(web::resource("/user").route(web::delete().to(delete_user::delete_user)))
            // Dockerflow Endpoints
            .service(
                web::resource("/__lbheartbeat__").route(web::get().to(dockerflow::lbheartbeat)),
            )
            .service(web::resource("/__heartbeat__").route(web::get().to(dockerflow::heartbeat)))
            .service(web::resource("/__version__").route(web::get().to(dockerflow::version)));

        if debug {
            app = app.service(web::resource("/debug").route(web::get().to(debug::debug_handler)));
        }

        app
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}
