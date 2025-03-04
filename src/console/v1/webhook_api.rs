use crate::common::constant::SEQ_NOTIFY_CONFIG_ID;
use crate::common::model::ApiResult;
use crate::common::share_data::ShareData;
use crate::console::model::webhook_model::{NotifyConfigAdd, NotifyConfigQuery, NotifyConfigUpdate};
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::sequence::{SequenceRequest, SequenceResult};
use crate::webhook::actor_model::{WebhookManagerReq, WebhookManagerResult};
use crate::webhook::model::{ChannelType, EmailType, NotifyConfigModelOb, WebHookSource};
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::collections::HashMap;
use std::sync::Arc;
use strum::IntoEnumIterator;

//选择下拉框借口
pub(crate) async fn query_notify_channel(
    _: Data<Arc<ShareData>>
) -> impl Responder {
    let mut map = HashMap::new();
    for channel_type in ChannelType::iter() {
        match &channel_type {
            ChannelType::WebHook(wh) => {
                for source in WebHookSource::iter() {
                    map.entry(format!("{}", channel_type))
                        .and_modify(|e: &mut Vec<String>| e.push(format!("{}", source)))
                        .or_insert(vec![format!("{}", source)])
                    ;
                }
            }
            ChannelType::Email(em) => {
                for e in EmailType::iter() {
                    map.entry(format!("{}", channel_type))
                        .and_modify(|v: &mut Vec<String>| v.push(format!("{}", e)))
                        .or_insert(vec![format!("{}", e)])
                    ;
                }
            }
            _ => {}
        }
    }
    HttpResponse::Ok().json(ApiResult::success(Some(map)))
}

pub(crate) async fn notify_config_add(
    share_data: Data<Arc<ShareData>>,
    web::Json(request): web::Json<NotifyConfigAdd>,
) -> impl Responder {
    let param = match request.to_param() {
        Ok(oj) => {
            oj
        }
        Err(_e) => {
            return HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some("notify_config_add error".to_string()),
            ));
        }
    };
    if let Ok(Ok(SequenceResult::NextId(id))) = share_data
        .sequence_manager
        .send(SequenceRequest::GetNextId(SEQ_NOTIFY_CONFIG_ID.clone()))
        .await
    {
        if let Ok(Ok(WebhookManagerResult::ConfigInfo(Some(info)))) = share_data
            .webhook_manager
            .send(WebhookManagerReq::AddConfig(NotifyConfigModelOb{ id, model:  param}))
            .await
        {
            HttpResponse::Ok().json(ApiResult::success(Some(info)))
        } else {
            HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some("notify_config_add error".to_string()),
            ))
        }
    }else{
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("notify_config_add error".to_string()),
        ))
    }
}

pub(crate) async fn notify_config_update(
    share_data: Data<Arc<ShareData>>,
    web::Json(request): web::Json<NotifyConfigUpdate>,
) -> impl Responder {
    let param = match request.to_param() {
        Ok(oj) => {
            oj
        }
        Err(_e) => {
            return HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some("notify_config_update error".to_string()),
            ));
        }
    };
    if let Ok(Ok(WebhookManagerResult::None)) = share_data
        .webhook_manager
        .send(WebhookManagerReq::UpdateConfig(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("notify_config_update error".to_string()),
        ))
    }
}

pub(crate) async fn notify_config_remove(
    share_data: Data<Arc<ShareData>>,
    web::Json(request): web::Json<u64>,
) -> impl Responder {
    let param = request;
    if let Ok(Ok(WebhookManagerResult::None)) = share_data
        .webhook_manager
        .send(WebhookManagerReq::RemoveConfig(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("notify_config_remove error".to_string()),
        ))
    }
}

pub(crate) async fn query_config_info(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<u64>,
) -> impl Responder {
    let param = request;
    if let Ok(Ok(WebhookManagerResult::ConfigInfo(info))) = share_data
        .webhook_manager
        .send(WebhookManagerReq::QueryConfig(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(info))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_config_info error".to_string()),
        ))
    }
}

pub(crate) async fn query_config_page(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<NotifyConfigQuery>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(WebhookManagerResult::ConfigInfo(info))) = share_data
        .webhook_manager
        .send(WebhookManagerReq::QueryConfigPage(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(info))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_config_page error".to_string()),
        ))
    }
}
