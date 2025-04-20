pub mod app_api;
pub mod cluster_api;
pub mod job_api;
pub mod login_api;
pub mod metrics_api;
pub mod namespace_api;
pub mod user_api;

use actix_web::web;
use actix_web::web::ServiceConfig;

pub const ERROR_CODE_SYSTEM_ERROR: &str = "SYSTEM_ERROR";
pub const ERROR_CODE_NO_PERMISSION: &str = "NO_PERMISSION";
pub const ERROR_CODE_NO_APP_PERMISSION: &str = "NO_APP_PERMISSION";

pub fn console_api_v1(config: &mut ServiceConfig) {
    config.service(
        web::scope("/ratchjob/api/console/v1")
            .service(
                web::resource("/namespaces/list")
                    .route(web::get().to(namespace_api::query_namespace_list)),
            )
            .service(web::resource("/login/login").route(web::post().to(login_api::login)))
            .service(web::resource("/login/captcha").route(web::get().to(login_api::gen_captcha)))
            .service(web::resource("/login/logout").route(web::post().to(login_api::logout)))
            .service(web::resource("/user/info").route(web::get().to(user_api::get_user_info)))
            .service(web::resource("/user/list").route(web::get().to(user_api::get_user_page_list)))
            .service(web::resource("/user/add").route(web::post().to(user_api::add_user)))
            .service(web::resource("/user/update").route(web::post().to(user_api::update_user)))
            .service(web::resource("/user/remove").route(web::post().to(user_api::remove_user)))
            .service(
                web::resource("/user/reset_password")
                    .route(web::post().to(user_api::reset_password)),
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
            .service(web::resource("/job/trigger").route(web::post().to(job_api::trigger_job)))
            .service(
                web::resource("/job/task/list").route(web::get().to(job_api::query_job_task_logs)),
            )
            .service(
                web::resource("/job/task/latest-history")
                    .route(web::get().to(job_api::query_latest_task)),
            )
            .service(
                web::resource("/metrics/timeline")
                    .route(web::get().to(metrics_api::query_metrics_timeline))
                    .route(web::post().to(metrics_api::query_metrics_timeline_json)),
            )
            .service(
                web::resource("/cluster/cluster_node_list")
                    .route(web::get().to(cluster_api::query_cluster_info)),
            ),
    );
}
