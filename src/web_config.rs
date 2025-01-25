use crate::common::app_config::AppConfig;
use crate::common::get_app_version;
use crate::openapi::openapi_config;
use crate::openapi::xxljob::xxl_api_config;
use actix_web::web::ServiceConfig;
use actix_web::Responder;
use std::sync::Arc;

pub async fn about_info() -> impl Responder {
    format!("ratch-job version:{}", get_app_version())
}

pub fn app_config(app_config: Arc<AppConfig>) -> impl FnOnce(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        openapi_config(config);
        xxl_api_config(config, &app_config);
    }
}
