pub mod v1;
pub mod xxljob;

use crate::openapi::v1::v1_api_config;
use crate::web_config::about_info;
use actix_web::web;
use actix_web::web::ServiceConfig;

pub fn openapi_config(config: &mut ServiceConfig) {
    config.service(web::resource("/api/about").route(web::get().to(about_info)));
    v1_api_config(config);
}
