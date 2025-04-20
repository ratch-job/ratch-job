use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::Arc;

use crate::cache::actor_model::{
    CacheManagerLocalReq, CacheManagerRaftReq, CacheManagerRaftResult, SetInfo,
};
use crate::cache::model::{CacheKey, CacheType, CacheValue};
use crate::common::datetime_utils::{now_millis_i64, now_second_i32, now_second_u32};
use crate::common::model::{ApiResult, UserSession};
use crate::common::share_data::ShareData;
use crate::raft::store::ClientRequest;
use crate::user::actor_model::{UserManagerRaftResult, UserManagerReq};
use crate::user::model::{UserDto, UserInfo};
use crate::user::permission::{UserRole, UserRoleHelper};
use actix_http::{HttpMessage, StatusCode};
use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use regex::Regex;

lazy_static::lazy_static! {
    pub static ref IGNORE_CHECK_LOGIN: Vec<&'static str> = vec![
        "/ratchjob/p/login", "/ratchjob/404",
        "/ratchjob/api/console/v1/login/login", "/ratchjob/api/console/v1/login/captcha",
    ];
    pub static ref STATIC_FILE_PATH: Regex= Regex::new(r"(?i).*\.(js|css|png|jpg|jpeg|bmp|svg)").unwrap();
    pub static ref API_PATH: Regex = Regex::new(r"(?i)/api/.*").unwrap();
}

#[derive(Clone)]
pub struct CheckLogin {
    app_share_data: Arc<ShareData>,
}

impl CheckLogin {
    pub fn new(app_share_data: Arc<ShareData>) -> Self {
        Self { app_share_data }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CheckLogin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckLoginMiddleware {
            service: Arc::new(service),
            app_share_data: self.app_share_data.clone(),
        }))
    }
}

#[derive(Clone)]
pub struct CheckLoginMiddleware<S> {
    service: Arc<S>,
    app_share_data: Arc<ShareData>,
}

impl<S, B> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let path = request.path();
        let is_check_path = !IGNORE_CHECK_LOGIN.contains(&path) && !STATIC_FILE_PATH.is_match(path);
        let is_page = !API_PATH.is_match(path);
        let token = if let Some(ck) = request.cookie("token") {
            ck.value().to_owned()
        } else if let Some(v) = request.headers().get("Token") {
            v.to_str().unwrap_or_default().to_owned()
        } else {
            "".to_owned()
        };
        let token = Arc::new(token);
        let app_share_data = self.app_share_data.clone();
        //request.parts()
        //let (http_request, _pl) = request.parts();
        //let http_request = http_request.to_owned();
        //let res = self.service.call(request);

        let service = self.service.clone();
        Box::pin(async move {
            let mut is_login = true;
            let mut user_has_permission = true;
            let path = request.path();
            let method = request.method().as_str();
            if is_check_path {
                is_login = if token.is_empty() {
                    false
                } else if let Ok(Some(session)) =
                    get_user_session(&app_share_data, token.clone()).await
                {
                    if session.roles.is_empty() {
                        user_has_permission = UserRoleHelper::match_url_by_base(path, method);
                    } else {
                        user_has_permission =
                            UserRole::match_url_by_roles(&session.roles, path, method);
                    }
                    request.extensions_mut().insert(session);
                    true
                } else {
                    false
                };
            }
            //log::info!("token: {}|{}|{}|{}|{}|{}",&token,is_page,is_check_path,is_login,request.path(),request.query_string());
            if is_login {
                if user_has_permission {
                    let res = service.call(request);
                    // forwarded responses map to "left" body
                    res.await.map(ServiceResponse::map_into_left_body)
                } else {
                    //已登录没有权限
                    let response = if is_page {
                        let move_url = format!("/ratchjob/nopermission?path={}", request.path());
                        HttpResponse::Ok()
                            .insert_header(("Location", move_url))
                            .status(StatusCode::FOUND)
                            .finish()
                            .map_into_right_body()
                    } else {
                        HttpResponse::Ok()
                            .insert_header(("No-Permission", "1"))
                            .json(ApiResult::<()>::error("NO_PERMISSION".to_owned(), None))
                            .map_into_right_body()
                    };
                    let (http_request, _pl) = request.into_parts();
                    let res = ServiceResponse::new(http_request, response);
                    Ok(res)
                }
            } else {
                //没有登录
                let response = if is_page {
                    let move_url =
                        if request.path() == "/ratchjob/p/login" || request.path() == "/p/login" {
                            format!("{}?{}", request.path(), request.query_string())
                        } else {
                            let mut redirect_param = HashMap::new();
                            if !request.query_string().is_empty() {
                                redirect_param.insert(
                                    "redirect_url",
                                    format!("{}?{}", request.path(), request.query_string()),
                                );
                            } else {
                                redirect_param.insert("redirect_url", request.path().to_owned());
                            };
                            let redirect_param =
                                serde_urlencoded::to_string(&redirect_param).unwrap_or_default();
                            format!("/ratchjob/p/login?{}", redirect_param)
                        };
                    HttpResponse::Ok()
                        .insert_header(("Location", move_url))
                        .status(StatusCode::FOUND)
                        .finish()
                        .map_into_right_body()
                } else {
                    HttpResponse::Ok()
                        .insert_header(("No-Login", "1"))
                        .json(ApiResult::<()>::error("NO_LOGIN".to_owned(), None))
                        .map_into_right_body()
                };
                let (http_request, _pl) = request.into_parts();
                let res = ServiceResponse::new(http_request, response);
                Ok(res)
            }
        })
    }
}

async fn get_user_session(
    app_share_data: &ShareData,
    token: Arc<String>,
) -> anyhow::Result<Option<Arc<UserSession>>> {
    let cache_key = CacheKey::new(CacheType::UserSession, token);
    let req = CacheManagerLocalReq::Get(cache_key.clone());
    match app_share_data.cache_manager.send(req).await?? {
        CacheManagerRaftResult::Value(CacheValue::UserSession(session)) => {
            let now = now_second_u32();
            if now < session.refresh_time + 5 {
                //更新5秒内不判断更新,避免频繁更新
                return Ok(Some(session));
            }
            let username = session.username.clone();
            match app_share_data
                .user_manager
                .send(UserManagerReq::Query { name: username })
                .await??
            {
                UserManagerRaftResult::QueryUser(Some(user)) => {
                    let refresh_time = (session.refresh_time as i64) * 1000;
                    if user.gmt_modified > refresh_time {
                        let ttl = (std::cmp::max((now as i64) * 1000 - refresh_time, 10000) / 1000)
                            as i32;
                        let new_session = build_user_session(user);
                        let set_info = SetInfo {
                            key: cache_key,
                            value: CacheValue::UserSession(new_session.clone()),
                            ttl,
                            now: now as i32,
                            nx: false,
                            xx: false,
                        };
                        let req = CacheManagerRaftReq::Set(set_info);
                        //todo 这里更新缓存可以考虑切换为异步执行
                        app_share_data
                            .raft_request_route
                            .request(ClientRequest::CacheReq { req })
                            .await
                            .ok();
                        return Ok(Some(new_session));
                    }
                }
                _ => {}
            };
            Ok(Some(session))
        }
        _ => Ok(None),
    }
}

fn build_user_session(user: UserInfo) -> Arc<UserSession> {
    Arc::new(UserSession {
        username: user.username,
        nickname: user.nickname,
        roles: user.roles,
        namespace_privilege: user.namespace_privilege,
        app_privilege: user.app_privilege,
        extend_infos: user.extend_info,
        refresh_time: now_second_i32() as u32,
    })
}
