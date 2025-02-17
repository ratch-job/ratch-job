pub mod app_api;
pub mod job_api;
pub mod namespace_api;
pub mod user_api;

use actix_web::web;
use actix_web::web::ServiceConfig;

pub const ERROR_CODE_SYSTEM_ERROR: &str = "SYSTEM_ERROR";

pub fn console_api_v1(config: &mut ServiceConfig) {
    config.service(
        web::scope("/ratchjob/api/console/v1")
            .service(
                web::resource("/namespaces/list")
                    .route(web::get().to(namespace_api::query_namespace_list)),
            )
            .service(
                web::resource("/user/web_resources")
                    .route(web::get().to(user_api::get_user_web_resources)),
            )
            .service(web::resource("/app/list").route(web::get().to(app_api::query_app_list)))
            .service(web::resource("/app/info").route(web::get().to(app_api::query_app_info)))
            .service(web::resource("/app/update").route(web::post().to(app_api::set_app)))
            .service(web::resource("/app/remove").route(web::post().to(app_api::remove_app)))
            .service(web::resource("/job/list").route(web::get().to(job_api::query_job_list)))
            .service(web::resource("/job/info").route(web::get().to(job_api::query_job_info)))
            .service(web::resource("/job/create").route(web::post().to(job_api::create_job)))
            .service(web::resource("/job/update").route(web::post().to(job_api::update_job)))
            .service(web::resource("/job/remove").route(web::post().to(job_api::remove_job)))
            .service(
                web::resource("/job/task/list").route(web::get().to(job_api::query_job_task_logs)),
            ),
    );
}
