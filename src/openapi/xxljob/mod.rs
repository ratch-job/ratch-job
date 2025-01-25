use crate::common::app_config::AppConfig;
use actix_web::web;
use actix_web::web::ServiceConfig;
use std::sync::Arc;

pub mod model;
pub mod server_api;

pub fn xxl_api_config(config: &mut ServiceConfig, app_config: &Arc<AppConfig>) {
    if app_config.xxl_job_prefix_path.is_empty() {
        config
            .service(web::resource("/api/registry").route(web::post().to(server_api::register)))
            .service(
                web::resource("/api/registryRemove").route(web::post().to(server_api::unregister)),
            )
            .service(web::resource("/api/callback").route(web::post().to(server_api::callback)));
    } else {
        config.service(
            web::scope(app_config.xxl_job_prefix_path.as_str())
                .service(web::resource("/api/registry").route(web::post().to(server_api::register)))
                .service(
                    web::resource("/api/registryRemove")
                        .route(web::post().to(server_api::unregister)),
                )
                .service(
                    web::resource("/api/callback").route(web::post().to(server_api::callback)),
                ),
        );
    }
}
