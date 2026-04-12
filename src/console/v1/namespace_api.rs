use crate::common::model::ApiResult;
use crate::common::share_data::ShareData;
use crate::common::string_utils::StringUtils;
use crate::console::model::namespace_model::{NamespaceHandleRequest, NamespaceInfo};
use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::namespace::model::actor_model::{
    NamespaceManagerRaftReq, NamespaceManagerRaftResult, NamespaceManagerReq,
    NamespaceManagerResult,
};
use crate::raft::store::ClientRequest;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub async fn query_namespace_list(app: Data<Arc<ShareData>>) -> impl Responder {
    let req = NamespaceManagerReq::QueryList;
    match app.namespace_manager.send(req).await {
        Ok(Ok(result)) => match result {
            NamespaceManagerResult::NamespaceList(list) => {
                let list: Vec<NamespaceInfo> = list
                    .into_iter()
                    .map(|ns| NamespaceInfo {
                        namespace_id: Some(ns.id.clone()),
                        namespace_name: Some(ns.name.clone()),
                        r#type: Some(ns.r#type.clone()),
                    })
                    .collect();
                HttpResponse::Ok().json(ApiResult::success(Some(list)))
            }
            _ => HttpResponse::Ok().json(ApiResult::<()>::error(
                "QUERY_NAMESPACE_ERROR".to_string(),
                Some("Invalid result type".to_string()),
            )),
        },
        Ok(Err(e)) => HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_string(),
            Some(e.to_string()),
        )),
        Err(_) => HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_string(),
            Some("Mailbox error".to_string()),
        )),
    }
}

pub async fn query_namespace_info(
    app: Data<Arc<ShareData>>,
    web::Query(id): web::Query<String>,
) -> impl Responder {
    let req = NamespaceManagerReq::GetNamespace(Arc::new(id));
    match app.namespace_manager.send(req).await {
        Ok(Ok(result)) => match result {
            NamespaceManagerResult::NamespaceInfo(Some(ns)) => {
                let info = NamespaceInfo {
                    namespace_id: Some(ns.id.clone()),
                    namespace_name: Some(ns.name.clone()),
                    r#type: Some(ns.r#type.clone()),
                };
                HttpResponse::Ok().json(ApiResult::success(Some(info)))
            }
            NamespaceManagerResult::NamespaceInfo(None) => {
                HttpResponse::Ok().json(ApiResult::<()>::error(
                    "NAMESPACE_NOT_FOUND".to_string(),
                    Some("Namespace not found".to_string()),
                ))
            }
            _ => HttpResponse::Ok().json(ApiResult::<()>::error(
                "QUERY_NAMESPACE_ERROR".to_string(),
                Some("Invalid result type".to_string()),
            )),
        },
        Ok(Err(e)) => HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_string(),
            Some(e.to_string()),
        )),
        Err(_) => HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_string(),
            Some("Mailbox error".to_string()),
        )),
    }
}

pub async fn update_namespace(
    app: Data<Arc<ShareData>>,
    web::Json(mut param): web::Json<NamespaceHandleRequest>,
) -> impl Responder {
    if StringUtils::is_option_empty(&param.namespace_name) {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace name cannot be empty".to_string()),
        ));
    }
    if StringUtils::is_option_empty_arc(&param.namespace_id) {
        param.namespace_id = Some(Arc::new(uuid::Uuid::new_v4().to_string()));
    }
    let id = param.namespace_id.clone();

    let msg = NamespaceManagerRaftReq::UpdateNamespace(param.to_param());
    match app
        .raft_request_route
        .request(ClientRequest::NamespaceReq { req: msg })
        .await
    {
        Ok(resp) => match resp {
            crate::raft::store::ClientResponse::NamespaceResp { resp } => match resp {
                NamespaceManagerRaftResult::None => {
                    HttpResponse::Ok().json(ApiResult::success(Some(id)))
                }
            },
            _ => HttpResponse::Ok().json(ApiResult::<()>::error(
                "UPDATE_NAMESPACE_ERROR".to_string(),
                Some("Invalid response type".to_string()),
            )),
        },
        Err(e) => HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_string(),
            Some(e.to_string()),
        )),
    }
}

pub async fn remove_namespace(
    app: Data<Arc<ShareData>>,
    web::Json(param): web::Json<NamespaceInfo>,
) -> impl Responder {
    if StringUtils::is_option_empty_arc(&param.namespace_id) {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace id cannot be empty".to_string()),
        ));
    }
    let id = param.namespace_id.unwrap();

    match app
        .job_manager
        .send(JobManagerReq::CountJobsByNamespace(id.as_str().to_string()))
        .await
    {
        Ok(Ok(JobManagerResult::Count(count))) => {
            if count > 0 {
                return HttpResponse::Ok().json(ApiResult::<()>::error(
                    "NAMESPACE_HAS_JOBS".to_string(),
                    Some(format!("Namespace has {} jobs, cannot delete", count)),
                ));
            }
        }
        _ => {
            log::error!("Failed to check jobs for namespace: {}", id);
            return HttpResponse::Ok().json(ApiResult::<()>::error(
                "SYSTEM_ERROR".to_string(),
                Some("Failed to check namespace dependencies".to_string()),
            ));
        }
    }

    let msg = NamespaceManagerRaftReq::Remove(id);
    match app
        .raft_request_route
        .request(ClientRequest::NamespaceReq { req: msg })
        .await
    {
        Ok(resp) => match resp {
            crate::raft::store::ClientResponse::NamespaceResp { resp: _ } => {
                HttpResponse::Ok().json(ApiResult::success(Some(true)))
            }
            _ => HttpResponse::Ok().json(ApiResult::<()>::error(
                "REMOVE_NAMESPACE_ERROR".to_string(),
                Some("Invalid response type".to_string()),
            )),
        },
        Err(e) => HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_string(),
            Some(e.to_string()),
        )),
    }
}
