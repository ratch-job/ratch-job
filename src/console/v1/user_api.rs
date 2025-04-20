use crate::common::constant::EMPTY_STR;
use crate::common::datetime_utils::now_millis_i64;
use crate::common::get_app_version;
use crate::common::model::{ApiResult, PageResult, UserSession};
use crate::common::share_data::ShareData;
use crate::console::model::user_model::{
    ResetPasswordParam, UpdateUserInfoParam, UserPageParams, UserPermissions, UserSimpleVO, UserVO,
};
use crate::raft::store::ClientRequest;
use crate::user::actor_model::{UserManagerRaftReq, UserManagerRaftResult, UserManagerReq};
use crate::user::model::UserDto;
use crate::user::permission::UserRole;
use actix_http::HttpMessage;
use actix_web::web::Data;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

///
/// 获取用户权限资源列表
/// 这里把取不到UserSession当成旧控制台，后继可以考虑单独实现一个接口
pub async fn get_user_web_resources(req: HttpRequest) -> actix_web::Result<impl Responder> {
    if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        let resources = if session.roles.is_empty() {
            UserRole::get_base_resources()
        } else {
            UserRole::get_web_resources_by_roles(session.roles.iter().map(|e| e.as_str()).collect())
        };
        let data = UserPermissions {
            resources,
            from: EMPTY_STR,
            version: get_app_version(),
            username: Some(session.username.clone()),
        };
        Ok(HttpResponse::Ok().json(ApiResult::success(Some(data))))
    } else {
        let data = UserPermissions {
            resources: vec![],
            from: "OLD_CONSOLE",
            version: get_app_version(),
            username: None,
        };
        Ok(HttpResponse::Ok().json(ApiResult::success(Some(data))))
    }
}

pub async fn get_user_info(req: HttpRequest) -> actix_web::Result<impl Responder> {
    if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        let userinfo = UserSimpleVO {
            username: Some(session.username.clone()),
            nickname: Some(session.nickname.clone()),
        };
        Ok(HttpResponse::Ok().json(ApiResult::success(Some(userinfo))))
    } else {
        Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "NOT_FOUND_USER_SESSION".to_owned(),
            None,
        )))
    }
}

pub async fn reset_password(
    req: HttpRequest,
    app: Data<Arc<ShareData>>,
    web::Json(param): web::Json<ResetPasswordParam>,
) -> actix_web::Result<impl Responder> {
    let (msg, username) = if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        let username = Arc::new(session.username.to_string());
        (
            UserManagerReq::CheckUser {
                name: username.clone(),
                password: param.old_password,
            },
            username,
        )
    } else {
        return Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "NOT_FOUND_USER_SESSION".to_owned(),
            None,
        )));
    };
    if let Ok(Ok(v)) = app.user_manager.send(msg).await {
        match v {
            UserManagerRaftResult::CheckUser(valid, _user) => {
                if valid {
                    let user_dto = UserDto {
                        username: username.clone(),
                        password: Some(param.new_password),
                        gmt_modified: Some(now_millis_i64()),
                        ..Default::default()
                    };
                    let msg = UserManagerRaftReq::UpdateUser(user_dto);
                    if let Ok(_) = app
                        .raft_request_route
                        .request(ClientRequest::UserReq { req: msg })
                        .await
                    {
                        return Ok(HttpResponse::Ok().json(ApiResult::success(Some(true))));
                    }
                }
            }
            _ => {
                return Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
                    "OLD_PASSWORD_INVALID".to_owned(),
                    None,
                )))
            }
        }
    }
    Ok(HttpResponse::Ok().json(ApiResult::<()>::error("SYSTEM_ERROR".to_owned(), None)))
}

pub async fn get_user_page_list(
    app: Data<Arc<ShareData>>,
    web::Query(param): web::Query<UserPageParams>,
) -> actix_web::Result<impl Responder> {
    let param = param.into();
    let req = UserManagerReq::QueryPageList(param);
    match app.user_manager.send(req).await.unwrap().unwrap() {
        UserManagerRaftResult::UserPage(total_count, list) => {
            let list: Vec<UserVO> = list.into_iter().map(|u| u.into()).collect();
            Ok(HttpResponse::Ok().json(ApiResult::success(Some(PageResult { total_count, list }))))
        }
        _ => Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "QUERY_USER_PAGE_ERROR".to_owned(),
            Some("result type is error".to_owned()),
        ))),
    }
}

pub async fn add_user(
    app: Data<Arc<ShareData>>,
    web::Json(user_param): web::Json<UpdateUserInfoParam>,
) -> actix_web::Result<impl Responder> {
    let user: UserDto = user_param.into();
    if user.roles.is_none() {
        return Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "USER_ROLE_IS_EMPTY".to_string(),
            Some("user roles is empty".to_owned()),
        )));
    }
    let msg = UserManagerRaftReq::AddUser(user);
    app.raft_request_route
        .request(ClientRequest::UserReq { req: msg })
        .await
        .ok();
    Ok(HttpResponse::Ok().json(ApiResult::success(Some(true))))
}

pub async fn update_user(
    app: Data<Arc<ShareData>>,
    web::Json(user_param): web::Json<UpdateUserInfoParam>,
) -> actix_web::Result<impl Responder> {
    let mut user: UserDto = user_param.into();
    user.gmt_create = None;
    if user.roles.is_none() {
        return Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "USER_ROLE_IS_EMPTY".to_string(),
            Some("user roles is empty".to_owned()),
        )));
    }
    let msg = UserManagerRaftReq::UpdateUser(user);
    app.raft_request_route
        .request(ClientRequest::UserReq { req: msg })
        .await
        .ok();
    Ok(HttpResponse::Ok().json(ApiResult::success(Some(true))))
}

pub async fn remove_user(
    app: Data<Arc<ShareData>>,
    web::Json(user): web::Json<UpdateUserInfoParam>,
) -> actix_web::Result<impl Responder> {
    let msg = UserManagerRaftReq::Remove(user.username);
    match app
        .raft_request_route
        .request(ClientRequest::UserReq { req: msg })
        .await
    {
        Ok(r) => Ok(HttpResponse::Ok().json(ApiResult::success(Some(true)))),
        Err(e) => Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_owned(),
            Some(e.to_string()),
        ))),
    }
}
