pub mod xxljob;

use crate::common::app_config::AppConfig;
use crate::web_config::about_info;
use actix_web::web;
use actix_web::web::ServiceConfig;

pub fn openapi_config(config: &mut ServiceConfig) {
    config.service(web::resource("/api/about").route(web::get().to(about_info)));
}
