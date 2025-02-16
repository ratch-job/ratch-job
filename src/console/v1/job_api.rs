use crate::common::model::{ApiResult, PageResult};
use crate::common::share_data::ShareData;
use crate::console::model::job::{JobInfoParam, JobQueryListRequest, JobTaskLogQueryListRequest};
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub(crate) async fn query_job_list(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<JobQueryListRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(JobManagerResult::JobPageInfo(total_count, list))) = share_data
        .job_manager
        .send(JobManagerReq::QueryJob(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(PageResult { total_count, list })))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_job_list error".to_string()),
        ))
    }
}

pub(crate) async fn query_job_info(
    share_data: Data<Arc<ShareData>>,
    web::Query(param): web::Query<JobInfoParam>,
) -> impl Responder {
    let id = param.id.unwrap_or_default();
    if let Ok(Ok(JobManagerResult::JobInfo(info))) =
        share_data.job_manager.send(JobManagerReq::GetJob(id)).await
    {
        HttpResponse::Ok().json(ApiResult::success(info))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_job_info error".to_string()),
        ))
    }
}

pub(crate) async fn add_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    if let Ok(Ok(JobManagerResult::JobInfo(Some(info)))) = share_data
        .job_manager
        .send(JobManagerReq::AddJob(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(info)))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("set_job error".to_string()),
        ))
    }
}

pub(crate) async fn update_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    if let Ok(Ok(_)) = share_data
        .job_manager
        .send(JobManagerReq::UpdateJob(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("update_job error".to_string()),
        ))
    }
}

pub(crate) async fn remove_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobInfoParam>,
) -> impl Responder {
    let id = param.id.unwrap_or_default();
    if let Ok(Ok(_)) = share_data.job_manager.send(JobManagerReq::Remove(id)).await {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("remove_job error".to_string()),
        ))
    }
}

pub(crate) async fn query_job_task_logs(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<JobTaskLogQueryListRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(JobManagerResult::JobTaskLogPageInfo(total_count, list))) = share_data
        .job_manager
        .send(JobManagerReq::QueryJobTaskLog(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(PageResult { total_count, list })))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_job_task_logs error".to_string()),
        ))
    }
}
