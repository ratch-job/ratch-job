mod app_api;

use actix_web::web;
use actix_web::web::ServiceConfig;

pub const ERROR_CODE_SYSTEM_ERROR: &str = "SYSTEM_ERROR";

pub fn console_api_v1(config: &mut ServiceConfig) {
    config.service(
        web::scope("/ratchjob/api/console/v1")
            .service(web::resource("/app/list").route(web::get().to(app_api::query_app_list)))
            .service(web::resource("/app/update").route(web::post().to(app_api::set_app)))
            .service(web::resource("/app/remove").route(web::post().to(app_api::remove_app))),
    );
}
