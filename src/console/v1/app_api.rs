use crate::app::model::{AppKey, AppManagerReq, AppManagerResult};
use crate::common::model::{ApiResult, PageResult};
use crate::common::share_data::ShareData;
use crate::console::model::app::{AppInfoParam, AppQueryListRequest};
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub(crate) async fn query_app_list(
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<AppQueryListRequest>,
) -> impl Responder {
    let param = request.to_param();
    if let Ok(Ok(AppManagerResult::AppPageInfo(total_count, list))) = share_data
        .app_manager
        .send(AppManagerReq::QueryApp(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(PageResult { total_count, list })))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_app_list error".to_string()),
        ))
    }
}

pub(crate) async fn set_app(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<AppInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    if param.name.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("app name is empty!".to_string()),
        ));
    }
    if let Ok(Ok(_)) = share_data
        .app_manager
        .send(AppManagerReq::UpdateApp(param))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("set_app error".to_string()),
        ))
    }
}

pub(crate) async fn remove_app(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<AppInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    if let Ok(Ok(_)) = share_data
        .app_manager
        .send(AppManagerReq::RemoveApp(AppKey::new(
            param.name.clone(),
            param.namespace.clone(),
        )))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(Some(())))
    } else {
        let error_msg = format!("remove_app error,param:{:?}", &param);
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some(error_msg),
        ))
    }
}
