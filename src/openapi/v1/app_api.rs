use crate::app::model::{AppKey, AppManagerReq, AppManagerResult};
use crate::common::constant::DEFAULT_XXL_NAMESPACE;
use crate::common::model::ApiResult;
use crate::common::share_data::ShareData;
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::openapi::v1::model::app_model::AppQueryParam;
use crate::openapi::xxljob::model::{xxl_api_empty_success, XxlApiResult};
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub(crate) async fn query_app_instance_addrs(
    share_data: Data<Arc<ShareData>>,
    web::Query(param): web::Query<AppQueryParam>,
) -> impl Responder {
    let namespace = if let Some(namespace) = param.namespace {
        Arc::from(namespace)
    } else {
        DEFAULT_XXL_NAMESPACE.clone()
    };
    let app_key = AppKey::new(Arc::from(param.app_name.unwrap_or_default()), namespace);
    if let Ok(Ok(AppManagerResult::AppInstanceAddrs(addrs))) = share_data
        .app_manager
        .send(AppManagerReq::GetAppInstanceAddrs(app_key.clone()))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(addrs)))
    } else {
        let error_msg = format!("query_app_instance_addrs error,app_key:{:?}", &app_key);
        log::error!("{}", &error_msg);
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some(error_msg),
        ))
    }
}
