use crate::common::model::{ApiResult, PageResult};
use crate::common::share_data::ShareData;
use crate::console::model::namespace_model::NamespaceInfo;
use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::namespace::model::actor_model::{NamespaceManagerRaftReq, NamespaceManagerReq, NamespaceManagerResult, NamespaceManagerRaftResult};
use crate::namespace::model::namespace::{NamespaceParam, NamespaceQueryParam};
use crate::raft::store::ClientRequest;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub async fn query_namespace_list(
    app: Data<Arc<ShareData>>,
    web::Query(param): web::Query<NamespaceQueryParam>,
) -> impl Responder {
    let req = NamespaceManagerReq::QueryNamespace(param);
    match app.namespace_manager.send(req).await {
        Ok(Ok(result)) => match result {
                NamespaceManagerResult::NamespacePageInfo(total, list) => {
                let list: Vec<NamespaceInfo> = list
                    .into_iter()
                    .map(|ns| NamespaceInfo {
                        namespace_id: Some(ns.id.clone()),
                        namespace_name: Some(ns.name.clone()),
                        r#type: Some(ns.r#type.clone()),
                    })
                    .collect();
                HttpResponse::Ok().json(ApiResult::success(Some(PageResult {
                    total_count: total,
                    list,
                })))
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
    let req = NamespaceManagerReq::GetNamespace(id);
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

pub async fn create_namespace(
    app: Data<Arc<ShareData>>,
    web::Json(param): web::Json<NamespaceParam>,
) -> impl Responder {
    if param.name.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace name cannot be empty".to_string()),
        ));
    }
    if param.r#type.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace type cannot be empty".to_string()),
        ));
    }

    let msg = NamespaceManagerRaftReq::AddNamespace(param);
    match app
        .raft_request_route
        .request(ClientRequest::NamespaceReq { req: msg })
        .await
    {
        Ok(resp) => match resp {
            crate::raft::store::ClientResponse::NamespaceResp { resp } => match resp {
                NamespaceManagerRaftResult::NamespaceInfo(ns) => {
                    let info = NamespaceInfo {
                        namespace_id: Some(ns.id.clone()),
                        namespace_name: Some(ns.name.clone()),
                        r#type: Some(ns.r#type.clone()),
                    };
                    HttpResponse::Ok().json(ApiResult::success(Some(info)))
                }
                NamespaceManagerRaftResult::None => {
                    HttpResponse::Ok().json(ApiResult::success(Some(true)))
                }
            },
            _ => HttpResponse::Ok().json(ApiResult::<()>::error(
                "CREATE_NAMESPACE_ERROR".to_string(),
                Some("Invalid response type".to_string()),
            )),
        },
        Err(e) => HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_string(),
            Some(e.to_string()),
        )),
    }
}

pub async fn update_namespace(
    app: Data<Arc<ShareData>>,
    web::Json(param): web::Json<NamespaceParam>,
) -> impl Responder {
    if param.id.is_none() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace id cannot be empty".to_string()),
        ));
    }
    if param.name.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace name cannot be empty".to_string()),
        ));
    }
    if param.r#type.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace type cannot be empty".to_string()),
        ));
    }

    let msg = NamespaceManagerRaftReq::UpdateNamespace(param);
    match app
        .raft_request_route
        .request(ClientRequest::NamespaceReq { req: msg })
        .await
    {
        Ok(resp) => match resp {
            crate::raft::store::ClientResponse::NamespaceResp { resp } => match resp {
                NamespaceManagerRaftResult::NamespaceInfo(ns) => {
                    let info = NamespaceInfo {
                        namespace_id: Some(ns.id.clone()),
                        namespace_name: Some(ns.name.clone()),
                        r#type: Some(ns.r#type.clone()),
                    };
                    HttpResponse::Ok().json(ApiResult::success(Some(info)))
                }
                NamespaceManagerRaftResult::None => {
                    HttpResponse::Ok().json(ApiResult::success(Some(true)))
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
    web::Query(id): web::Query<String>,
) -> impl Responder {
    if id.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            "INVALID_PARAMETER".to_string(),
            Some("Namespace id cannot be empty".to_string()),
        ));
    }

    match app
        .job_manager
        .send(JobManagerReq::CountJobsByNamespace(id.clone()))
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
