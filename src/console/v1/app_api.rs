use crate::app::model::{AppKey, AppManagerRaftReq, AppManagerReq, AppManagerResult, RegisterType};
use crate::common::model::{ApiResult, PageResult, UserSession};
use crate::common::share_data::ShareData;
use crate::console::model::app::{AppInfoParam, AppQueryListRequest};
use crate::console::v1::{ERROR_CODE_NO_APP_PERMISSION, ERROR_CODE_SYSTEM_ERROR};
use crate::raft::store::ClientRequest;
use actix_http::HttpMessage;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub(crate) async fn query_app_list(
    req: actix_web::HttpRequest,
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<AppQueryListRequest>,
) -> impl Responder {
    let session = if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        session.clone()
    } else {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("user session is invalid".to_string()),
        ));
    };
    let param = request.to_param_with_session(&session);
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

pub(crate) async fn query_app_info(
    req: actix_web::HttpRequest,
    share_data: Data<Arc<ShareData>>,
    web::Query(param): web::Query<AppInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    if param.app_name.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("app name is empty!".to_string()),
        ));
    }
    let app_privilege = if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        session.app_privilege.clone()
    } else {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("user session is invalid".to_string()),
        ));
    };
    if !app_privilege.check_permission(&param.app_name) {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_NO_APP_PERMISSION.to_string(),
            Some(format!("user no app permission:{}", &param.app_name)),
        ));
    }
    if let Ok(Ok(AppManagerResult::AppInfo(info))) = share_data
        .app_manager
        .send(AppManagerReq::GetApp(param.build_app_key()))
        .await
    {
        HttpResponse::Ok().json(ApiResult::success(info))
    } else {
        HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("query_app_info error".to_string()),
        ))
    }
}

pub(crate) async fn set_app(
    req: actix_web::HttpRequest,
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<AppInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    if param.app_name.is_empty() {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("app name is empty!".to_string()),
        ));
    }
    let app_privilege = if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        session.app_privilege.clone()
    } else {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("user session is invalid".to_string()),
        ));
    };
    if !app_privilege.check_permission(&param.app_name) {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_NO_APP_PERMISSION.to_string(),
            Some(format!("user no app permission:{}", &param.app_name)),
        ));
    }
    if let Ok(_) = share_data
        .raft_request_route
        .request(ClientRequest::AppReq {
            req: AppManagerRaftReq::UpdateApp(param),
        })
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
    req: actix_web::HttpRequest,
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<AppInfoParam>,
) -> impl Responder {
    let param = param.to_param();
    let app_privilege = if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        session.app_privilege.clone()
    } else {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("user session is invalid".to_string()),
        ));
    };
    if !app_privilege.check_permission(&param.app_name) {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_NO_APP_PERMISSION.to_string(),
            Some(format!("user no app permission:{}", &param.app_name)),
        ));
    }
    if let Ok(Ok(AppManagerResult::AppInfo(Some(info)))) = share_data
        .app_manager
        .send(AppManagerReq::GetApp(param.build_app_key()))
        .await
    {
        let register_type = RegisterType::from_str(&info.register_type);
        let is_empty = if let Some(addrs) = info.instance_addrs {
            addrs.is_empty()
        } else {
            true
        };
        if register_type == RegisterType::Auto && !is_empty {
            return HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some("应用存在注册的实例，不允许删除!".to_string()),
            ));
        }
    } else {
        //不存在数据，相当与已删除
        return HttpResponse::Ok().json(ApiResult::success(Some(())));
    }
    if let Ok(_) = share_data
        .raft_request_route
        .request(ClientRequest::AppReq {
            req: AppManagerRaftReq::RemoveApp(param.build_app_key()),
        })
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
