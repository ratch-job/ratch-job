pub mod app_api;
pub mod job_api;
pub mod model;
pub mod raft_api;

use crate::web_config::about_info;
use actix_web::web;
use actix_web::web::ServiceConfig;

pub fn v1_api_config(config: &mut ServiceConfig) {
    // /api/v1 后续计划废弃
    config.service(
        web::scope("/api/v1")
            .service(
                web::resource("/app/instance/addrs")
                    .route(web::get().to(app_api::query_app_instance_addrs)),
            )
            .service(web::resource("/job/create").route(web::post().to(job_api::create_job)))
            .service(web::resource("/job/update").route(web::post().to(job_api::update_job)))
            .service(web::resource("/job/info").route(web::get().to(job_api::get_job_info)))
            .service(web::resource("/job/list").route(web::get().to(job_api::query_job_list)))
            .service(web::resource("/raft/metrics").route(web::get().to(raft_api::metrics)))
            .service(web::resource("/about").route(web::get().to(about_info))),
    );
    config.service(
        web::scope("/ratch/v1")
            .service(
                web::resource("/app/instance/addrs")
                    .route(web::get().to(app_api::query_app_instance_addrs)),
            )
            .service(web::resource("/job/create").route(web::post().to(job_api::create_job)))
            .service(web::resource("/job/update").route(web::post().to(job_api::update_job)))
            .service(web::resource("/job/remove").route(web::post().to(job_api::remove_job)))
            .service(web::resource("/job/info").route(web::get().to(job_api::get_job_info)))
            .service(web::resource("/job/list").route(web::get().to(job_api::query_job_list)))
            .service(web::resource("/raft/metrics").route(web::get().to(raft_api::metrics)))
            .service(web::resource("/about").route(web::get().to(about_info))),
    );
}
