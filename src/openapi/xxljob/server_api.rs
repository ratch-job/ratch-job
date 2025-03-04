use crate::app::model::{AppKey, AppManagerReq};
use crate::common::constant::DEFAULT_XXL_NAMESPACE;
use crate::common::share_data::ShareData;
use crate::openapi::xxljob::model::server_request::{CallbackParam, RegistryParam};
use crate::openapi::xxljob::model::{xxl_api_empty_success, XxlApiResult};
use crate::raft::store::ClientRequest;
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
    if let Ok(Ok(_)) = share_data
        .app_manager
        .send(AppManagerReq::RegisterAppInstance(
            app_key,
            instance_addr.clone(),
        ))
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
    if let Ok(Ok(_)) = share_data
        .app_manager
        .send(AppManagerReq::UnregisterAppInstance(
            app_key,
            instance_addr.clone(),
        ))
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
    web::Json(params): web::Json<Vec<CallbackParam>>,
) -> impl Responder {
    let id_list: Vec<u64> = params.iter().map(|p| p.log_id).collect();
    if let Ok(_) = share_data
        .raft_request_route
        .request(ClientRequest::ScheduleReq {
            req: ScheduleManagerRaftReq::TaskCallBacks(
                params.into_iter().map(|p| p.into()).collect(),
            ),
        })
        .await
    {
        log::info!("callback success,id list:{:?}", &id_list);
        HttpResponse::Ok().json(xxl_api_empty_success())
    } else {
        let error_msg = format!("callback error,id list:{:?}", &id_list);
        log::error!("{}", &error_msg);
        HttpResponse::Ok().json(XxlApiResult::<()>::fail(Some(error_msg)))
    }
}
