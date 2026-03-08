use crate::common::constant::SEQ_JOB_ID;
use crate::common::datetime_utils::now_millis;
use crate::common::model::{ApiResult, PageResult};
use crate::common::share_data::ShareData;
use crate::console::model::job::JobQueryListRequest;
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::job::model::actor_model::{
    JobManagerRaftReq, JobManagerRaftResult, JobManagerReq, JobManagerResult,
};
use crate::job::model::job::JobParam;
use crate::openapi::v1::model::job_model::{JobTaskHistoryRequest, JobTaskListRequest};
use crate::openapi::xxljob::model::XxlApiResult;
use crate::raft::store::{ClientRequest, ClientResponse};
use crate::schedule::model::actor_model::{ScheduleManagerReq, ScheduleManagerResult};
use crate::sequence::{SequenceRequest, SequenceResult};
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::future::Future;
use std::sync::Arc;

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
        param.update_time = Some(now_millis());
        if let ClientResponse::JobResp {
            resp: JobManagerRaftResult::JobInfo(job),
        } = share_data
            .raft_request_route
            .request(ClientRequest::JobReq {
                req: JobManagerRaftReq::AddJob(param),
            })
            .await?
        {
            Ok(HttpResponse::Ok().json(XxlApiResult::success(Some(job))))
        } else {
            Err(anyhow::anyhow!("create job result type error!"))
        }
    } else {
        Err(anyhow::anyhow!("get job id error!"))
    }
}
pub(crate) async fn create_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobParam>,
) -> impl Responder {
    match do_create_job(share_data, param).await {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!("create_job error,{}", e);
            log::error!("{}", &error_msg);
            HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some(error_msg),
            ))
        }
    }
}

async fn do_update_job(
    share_data: Data<Arc<ShareData>>,
    mut param: JobParam,
) -> anyhow::Result<HttpResponse> {
    param.update_time = Some(now_millis());
    share_data
        .raft_request_route
        .request(ClientRequest::JobReq {
            req: JobManagerRaftReq::UpdateJob(param),
        })
        .await?;
    Ok(HttpResponse::Ok().json(XxlApiResult::success(Some(()))))
}
pub(crate) async fn update_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobParam>,
) -> impl Responder {
    match do_update_job(share_data, param).await {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!("update_job error,{}", e);
            log::error!("{}", &error_msg);
            HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some(error_msg),
            ))
        }
    }
}

async fn do_remove_job(
    share_data: Data<Arc<ShareData>>,
    mut param: JobParam,
) -> anyhow::Result<HttpResponse> {
    let id = if let Some(id) = param.id {
        id
    } else {
        return Err(anyhow::anyhow!("job id is null"));
    };
    share_data
        .raft_request_route
        .request(ClientRequest::JobReq {
            req: JobManagerRaftReq::Remove(id),
        })
        .await?;
    Ok(HttpResponse::Ok().json(XxlApiResult::success(Some(()))))
}

pub(crate) async fn remove_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<JobParam>,
) -> impl Responder {
    match do_remove_job(share_data, param).await {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!("remove_job error,{}", e);
            log::error!("{}", &error_msg);
            HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some(error_msg),
            ))
        }
    }
}

pub(crate) async fn get_job_info(
    share_data: Data<Arc<ShareData>>,
    web::Query(param): web::Query<JobParam>,
) -> impl Responder {
    let id = param.id.unwrap_or_default();
    if let Ok(Ok(JobManagerResult::JobInfo(Some(job_info)))) =
        share_data.job_manager.send(JobManagerReq::GetJob(id)).await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(job_info)))
    } else {
        let error_msg = format!("get_job_info error,id:{}", id);
        log::error!("{}", &error_msg);
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some(error_msg),
        ))
    }
}

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

/// 导出任务列表（JSON 格式）
pub(crate) async fn export_jobs(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<JobQueryListRequest>,
) -> impl Responder {
    let mut param = request.to_param();
    param.offset = 0;
    param.limit = 0xffff_ffff;
    if let Ok(Ok(JobManagerResult::JobPageInfo(_total_count, list))) = share_data
        .job_manager
        .send(JobManagerReq::QueryJob(param))
        .await
    {
        HttpResponse::Ok()
            .insert_header(("Content-Disposition", "attachment; filename=\"jobs.json\""))
            .json(ApiResult::success(Some(list)))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("export_jobs error".to_string()),
        ))
    }
}

/// 导入任务列表（JSON 格式）
pub(crate) async fn import_jobs(
    share_data: Data<Arc<ShareData>>,
    web::Json(params): web::Json<Vec<JobParam>>,
) -> impl Responder {
    let mut success_count = 0u64;
    let mut fail_count = 0u64;
    let mut errors: Vec<String> = Vec::new();

    for mut param in params {
        // 为每个导入的任务分配新 id
        match share_data
            .sequence_manager
            .send(SequenceRequest::GetNextId(SEQ_JOB_ID.clone()))
            .await
        {
            Ok(Ok(SequenceResult::NextId(id))) => {
                param.id = Some(id);
                param.update_time = Some(now_millis());
                match share_data
                    .raft_request_route
                    .request(ClientRequest::JobReq {
                        req: JobManagerRaftReq::AddJob(param),
                    })
                    .await
                {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        fail_count += 1;
                        errors.push(format!("add job error: {}", e));
                    }
                }
            }
            _ => {
                fail_count += 1;
                errors.push("get job id error".to_string());
            }
        }
    }

    let msg = format!(
        "import done: {} success, {} failed",
        success_count, fail_count
    );
    if fail_count > 0 {
        HttpResponse::Ok().json(ApiResult::<String>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some(format!("{}, errors: {:?}", msg, errors)),
        ))
    } else {
        HttpResponse::Ok().json(ApiResult::success(Some(msg)))
    }
}

pub(crate) async fn query_job_task_list(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<JobTaskListRequest>,
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
            Some("query_job_task_list error".to_string()),
        ))
    }
}

pub(crate) async fn query_latest_task_history(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<JobTaskHistoryRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(ScheduleManagerResult::JobTaskLogPageInfo(total_count, list))) = share_data
        .schedule_manager
        .send(ScheduleManagerReq::QueryJobTaskLog(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(PageResult { total_count, list })))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_latest_task_history error".to_string()),
        ))
    }
}
