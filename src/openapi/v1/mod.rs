pub mod app_api;
pub mod job_api;
pub mod model;

use actix_web::web;
use actix_web::web::ServiceConfig;

pub fn v1_api_config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/api/v1").service(
            web::resource("/app/instance/addrs")
                .route(web::get().to(app_api::query_app_instance_addrs)),
        ),
    );
}
