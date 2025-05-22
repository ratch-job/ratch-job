use crate::app::model::{AppInstanceParam, AppKey, AppManagerRaftReq, AppManagerReq};
use crate::common::constant::DEFAULT_XXL_NAMESPACE;
use crate::common::datetime_utils::{now_millis_i64, now_second_u32};
use crate::common::share_data::ShareData;
use crate::openapi::xxljob::model::server_request::{CallbackParam, RegistryParam};
use crate::openapi::xxljob::model::{xxl_api_empty_success, XxlApiResult};
use crate::raft::store::ClientRequest;
use crate::schedule::batch_call::BatchCallManagerReq;
use crate::schedule::model::actor_model::ScheduleManagerRaftReq;
use crate::task::model::actor_model::TaskManagerReq;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use log::log;
use std::sync::Arc;

pub(crate) async fn register(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<RegistryParam>,
) -> impl Responder {
    let app_name = param.registry_key;
    let instance_addr = param.registry_value;
    let app_key = AppKey::new(app_name.clone(), DEFAULT_XXL_NAMESPACE.clone());
    let app_param = AppInstanceParam {
        app_key,
        instance_addr: instance_addr.clone(),
        last_modified_time: now_second_u32(),
    };
    if let Ok(_) = share_data
        .raft_request_route
        .request(ClientRequest::AppReq {
            req: AppManagerRaftReq::RegisterInstance(app_param),
        })
        .await
    {
        HttpResponse::Ok().json(xxl_api_empty_success())
    } else {
        let error_msg = format!(
            "register error,app_name:{},addr:{}",
            app_name, instance_addr
        );
        log::error!("{}", &error_msg);
        HttpResponse::Ok().json(XxlApiResult::<()>::fail(Some(error_msg)))
    }
}

pub(crate) async fn unregister(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<RegistryParam>,
) -> impl Responder {
    let app_name = param.registry_key;
    let instance_addr = param.registry_value;
    let app_key = AppKey::new(app_name.clone(), DEFAULT_XXL_NAMESPACE.clone());
    let app_param = AppInstanceParam {
        app_key,
        instance_addr: instance_addr.clone(),
        last_modified_time: now_second_u32(),
    };
    if let Ok(_) = share_data
        .raft_request_route
        .request(ClientRequest::AppReq {
            req: AppManagerRaftReq::UnregisterInstance(app_param),
        })
        .await
    {
        HttpResponse::Ok().json(xxl_api_empty_success())
    } else {
        let error_msg = format!(
            "unregister error,app_name:{},addr:{}",
            app_name, instance_addr
        );
        log::error!("{}", &error_msg);
        HttpResponse::Ok().json(XxlApiResult::<()>::fail(Some(error_msg)))
    }
}

pub(crate) async fn callback(
    share_data: Data<Arc<ShareData>>,
    web::Json(mut params): web::Json<Vec<CallbackParam>>,
) -> impl Responder {
    let now = now_millis_i64();
    #[cfg(feature = "debug")]
    log::info!("callback params:{:?}", &params);
    let id_list: Vec<u64> = params
        .iter_mut()
        .map(|p| {
            //回调时间定义是任务启动，这里统一设置为收到消息后的当前时间
            p.log_date_time = now;
            p.log_id
        })
        .collect();
    if let Ok(_) = share_data
        .batch_call_manager
        .send(BatchCallManagerReq::Callback(params))
        .await
    {
        #[cfg(feature = "debug")]
        log::info!("callback success,id list:{:?}", &id_list);
        HttpResponse::Ok().json(xxl_api_empty_success())
    } else {
        let error_msg = format!("callback error,id list:{:?}", &id_list);
        log::error!("{}", &error_msg);
        HttpResponse::Ok().json(XxlApiResult::<()>::fail(Some(error_msg)))
    }
}
