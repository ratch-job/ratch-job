use crate::common::constant::SEQ_JOB_ID;
use crate::common::model::{ApiResult, PageResult};
use crate::common::share_data::ShareData;
use crate::console::model::job::{JobInfoParam, JobQueryListRequest, JobTaskLogQueryListRequest};
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::job::model::job::JobParam;
use crate::sequence::{SequenceRequest, SequenceResult};
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
    if id == 0 {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_job_info error,the job id is invalid".to_string()),
        ));
    }
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

async fn do_create_job(
    share_data: Data<Arc<ShareData>>,
    mut param: JobParam,
) -> anyhow::Result<HttpResponse> {
    if let SequenceResult::NextId(id) = share_data
        .sequence_manager
        .send(SequenceRequest::GetNextId(SEQ_JOB_ID.clone()))
        .await??
    {
        param.id = Some(id);
        if let JobManagerResult::JobInfo(Some(job_info)) = share_data
            .job_manager
            .send(JobManagerReq::AddJob(param))
            .await??
        {
            Ok(HttpResponse::Ok().json(ApiResult::success(Some(job_info))))
        } else {
            Err(anyhow::anyhow!("create job result type error!"))
        }
    } else {
        Err(anyhow::anyhow!("get job id error!"))
    }
}

pub(crate) async fn create_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    match do_create_job(share_data, param).await {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!("create_job error,{}", e);
            //log::error!("{}", &error_msg);
            HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some(error_msg),
            ))
        }
    }
}

pub(crate) async fn update_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    let id = param.id.clone().unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("update_job error,the job id is invalid".to_string()),
        ));
    }
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
    if id == 0 {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("remove_job error,the job id is invalid".to_string()),
        ));
    }
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
