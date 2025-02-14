pub mod model;
pub mod v1;

use crate::console::v1::console_api_v1;
use crate::web_config::about_info;
use actix_web::web;
use actix_web::web::ServiceConfig;

pub fn console_config(config: &mut ServiceConfig) {
    console_page_config(config);
    console_api_v1(config);
}

pub fn console_page_config(config: &mut ServiceConfig) {
    config
        .service(web::resource("/").route(web::get().to(about_info)))
        .service(web::resource("/about").route(web::get().to(about_info)));
}
