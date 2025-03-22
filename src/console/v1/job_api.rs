use crate::common::constant::{EMPTY_ARC_STR, SEQ_JOB_ID};
use crate::common::datetime_utils::{now_millis, now_second_u32};
use crate::common::model::{ApiResult, PageResult};
use crate::common::share_data::ShareData;
use crate::console::model::job::{
    JobInfoParam, JobQueryListRequest, JobTaskLogQueryListRequest, TriggerJobParam,
};
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::job::model::actor_model::{
    JobManagerRaftReq, JobManagerRaftResult, JobManagerReq, JobManagerResult,
};
use crate::job::model::job::JobParam;
use crate::raft::store::{ClientRequest, ClientResponse};
use crate::schedule::model::actor_model::{ScheduleManagerReq, ScheduleManagerResult};
use crate::sequence::{SequenceRequest, SequenceResult};
use crate::task::model::actor_model::{
    TaskHistoryManagerReq, TaskHistoryManagerResult, TaskManagerReq, TriggerItem,
};
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
    param.check_valid()?;
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
            Ok(HttpResponse::Ok().json(ApiResult::success(Some(job))))
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
    let mut param = param.to_param();
    let id = param.id.clone().unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("update_job error,the job id is invalid".to_string()),
        ));
    }
    if let Err(e) = param.check_valid() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some(format!("update_job error,{}", e)),
        ));
    }
    if let Ok(_) = share_data
        .raft_request_route
        .request(ClientRequest::JobReq {
            req: JobManagerRaftReq::UpdateJob(param),
        })
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
    if let Ok(_) = share_data
        .raft_request_route
        .request(ClientRequest::JobReq {
            req: JobManagerRaftReq::Remove(id),
        })
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("remove_job error".to_string()),
        ))
    }
}

pub(crate) async fn trigger_job(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<TriggerJobParam>,
) -> impl Responder {
    let id = param.job_id.unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("trigger_job error,the job id is invalid".to_string()),
        ));
    }
    let job_info = if let Ok(Ok(JobManagerResult::JobInfo(Some(job_info)))) =
        share_data.job_manager.send(JobManagerReq::GetJob(id)).await
    {
        job_info
    } else {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_job_info error".to_string()),
        ));
    };
    let task_item = TriggerItem::new_with_user(
        now_second_u32(),
        job_info,
        param.instance_addr.unwrap_or(EMPTY_ARC_STR.clone()),
        EMPTY_ARC_STR.clone(),
    );
    log::info!("trigger_job task_item:{:?}", &task_item);
    if let Ok(Ok(_)) = share_data
        .task_manager
        .send(TaskManagerReq::TriggerTaskList(vec![task_item]))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("trigger_job error".to_string()),
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

pub(crate) async fn query_latest_task(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<JobTaskLogQueryListRequest>,
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
            Some("query_latest_task error".to_string()),
        ))
    }
}
