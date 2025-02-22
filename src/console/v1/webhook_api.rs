use crate::common::model::{ApiResult, PageResult};
use crate::common::share_data::ShareData;
use crate::console::model::webhook_model::{
    EventQueryListRequest, EventRemoveRequest, EventUpdateRequest, ObjectQueryListRequest,
    ObjectRemoveRequest, ObjectUpdateRequest,
};
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::webhook::actor_model::{WebhookManagerReq, WebhookManagerResult};
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub(crate) async fn query_object_list(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<ObjectQueryListRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(WebhookManagerResult::ObjectPageInfo(total_count, list))) = share_data
        .webhook_manager
        .send(WebhookManagerReq::QueryObjectPage(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(PageResult { total_count, list })))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_object_list error".to_string()),
        ))
    }
}

pub(crate) async fn webhook_object_update(
    share_data: Data<Arc<ShareData>>,
    web::Json(request): web::Json<ObjectUpdateRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(WebhookManagerResult::None)) = share_data
        .webhook_manager
        .send(WebhookManagerReq::UpdateObject(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("webhook_object_update error".to_string()),
        ))
    }
}

pub(crate) async fn webhook_object_remove(
    share_data: Data<Arc<ShareData>>,
    web::Json(request): web::Json<ObjectRemoveRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(WebhookManagerResult::None)) = share_data
        .webhook_manager
        .send(WebhookManagerReq::RemoveObject(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("webhook_object_remove error".to_string()),
        ))
    }
}

pub(crate) async fn query_event_list(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<EventQueryListRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(WebhookManagerResult::EventPageInfo(total_count, list))) = share_data
        .webhook_manager
        .send(WebhookManagerReq::QueryEventPage(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(PageResult { total_count, list })))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_event_list error".to_string()),
        ))
    }
}

pub(crate) async fn webhook_event_update(
    share_data: Data<Arc<ShareData>>,
    web::Json(request): web::Json<EventUpdateRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(WebhookManagerResult::None)) = share_data
        .webhook_manager
        .send(WebhookManagerReq::UpdateEvent(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("webhook_event_update error".to_string()),
        ))
    }
}

pub(crate) async fn webhook_event_remove(
    share_data: Data<Arc<ShareData>>,
    web::Json(request): web::Json<EventRemoveRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(WebhookManagerResult::None)) = share_data
        .webhook_manager
        .send(WebhookManagerReq::RemoveEvent(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("webhook_event_remove error".to_string()),
        ))
    }
}
