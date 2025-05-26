use std::sync::Arc;

use actix_web::http::header;
use actix_web::{
    cookie::Cookie,
    web::{self, Data},
    HttpRequest, HttpResponse, Responder,
};
use captcha::filters::{Grid, Noise};
use captcha::Captcha;

use crate::cache::actor_model::{
    CacheManagerLocalReq, CacheManagerRaftReq, CacheManagerRaftResult, SetInfo,
};
use crate::common::constant::CONSOLE_TOKEN_COOKIE_KEY;
use crate::common::datetime_utils::{now_second_i32, now_second_u32};
use crate::common::share_data::ShareData;
use crate::console::model::login_model::{LoginParam, LoginToken};
use crate::raft::store::ClientRequest;
use crate::user::actor_model::{UserManagerRaftResult, UserManagerReq};
use crate::{
    cache::model::{CacheKey, CacheType, CacheValue},
    common::{
        crypto_utils,
        model::{ApiResult, UserSession},
    },
};

pub async fn login(
    request: HttpRequest,
    app: Data<Arc<ShareData>>,
    web::Form(param): web::Form<LoginParam>,
) -> actix_web::Result<impl Responder> {
    let captcha_token = if let Some(ck) = request.cookie("ratch_captcha_token") {
        ck.value().to_owned()
    } else {
        String::new()
    };
    if app.app_config.console_captcha_enable {
        //校验验证码
        if captcha_token.is_empty() {
            return Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
                "CAPTCHA_CHECK_ERROR".to_owned(),
                Some("captcha token is empty".to_owned()),
            )));
        }
        let captcha_code = param.captcha.unwrap_or_default().to_uppercase();
        let cache_req = CacheManagerLocalReq::Get(CacheKey::new(
            CacheType::String,
            Arc::new(format!("Captcha_{}", &captcha_token)),
        ));
        let captcha_check_result =
            if let Ok(Ok(CacheManagerRaftResult::Value(CacheValue::String(v)))) =
                app.cache_manager.send(cache_req).await
            {
                &captcha_code == v.as_ref()
            } else {
                false
            };
        if !captcha_check_result {
            return Ok(HttpResponse::Ok()
                .cookie(
                    Cookie::build("ratch_captcha_token", "")
                        .path("/")
                        .http_only(true)
                        .finish(),
                )
                .json(ApiResult::<()>::error(
                    "CAPTCHA_CHECK_ERROR".to_owned(),
                    Some("CAPTCHA_CHECK_ERROR".to_owned()),
                )));
        }
    }

    //todo 补充登录限流

    let password = match decode_password(&param.password, &captcha_token) {
        Ok(v) => v,
        Err(e) => {
            log::error!("decode_password error:{}", e);
            return Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
                "SYSTEM_ERROR".to_owned(),
                Some("decode_password error".to_owned()),
            )));
        }
    };
    let msg = UserManagerReq::CheckUser {
        name: param.username,
        password,
    };
    if let Ok(Ok(res)) = app.user_manager.send(msg).await {
        if let UserManagerRaftResult::CheckUser(valid, user) = res {
            if valid {
                //增加长度避免遍历
                let token = Arc::new(
                    uuid::Uuid::new_v4().to_string().replace('-', "")
                        + &uuid::Uuid::new_v4().to_string().replace('-', ""),
                );
                let now = now_second_i32();
                let session = Arc::new(UserSession {
                    username: user.username,
                    nickname: user.nickname,
                    roles: user.roles,
                    extend_infos: user.extend_info,
                    namespace_privilege: user.namespace_privilege,
                    app_privilege: user.app_privilege,
                    refresh_time: now as u32,
                });
                let set_info = SetInfo {
                    key: CacheKey::new(CacheType::UserSession, token.clone()),
                    value: CacheValue::UserSession(session),
                    ttl: app.app_config.console_login_timeout,
                    now,
                    nx: false,
                    xx: false,
                };
                let cache_req = CacheManagerRaftReq::Set(set_info);
                app.raft_request_route
                    .request(ClientRequest::CacheReq { req: cache_req })
                    .await
                    .ok();
                //登录成功后清除登陆限流计数
                //let clear_limit_req =
                //    CacheManagerReq::Remove(CacheKey::new(CacheType::String, limit_key));
                //app.cache_manager.do_send(clear_limit_req);
                let login_token = LoginToken {
                    token: token.to_string(),
                };
                return Ok(HttpResponse::Ok()
                    .cookie(
                        Cookie::build(CONSOLE_TOKEN_COOKIE_KEY, token.as_str())
                            .path("/")
                            .http_only(true)
                            .finish(),
                    )
                    .cookie(
                        Cookie::build("ratch_captcha_token", "")
                            .path("/")
                            .http_only(true)
                            .finish(),
                    )
                    .insert_header(header::ContentType(mime::APPLICATION_JSON))
                    .json(ApiResult::success(Some(login_token))));
            }
        }
        return Ok(
            HttpResponse::Ok().json(ApiResult::<()>::error("USER_CHECK_ERROR".to_owned(), None))
        );
    }
    Ok(HttpResponse::Ok().json(ApiResult::<()>::error("SYSTEM_ERROR".to_owned(), None)))
}

fn decode_password(password: &str, captcha_token: &str) -> anyhow::Result<String> {
    let password_data = crypto_utils::decode_base64(password)?;
    if captcha_token.is_empty() {
        let password = String::from_utf8(password_data)?;
        Ok(password)
    } else {
        let password = String::from_utf8(crypto_utils::decrypt_aes128(
            &captcha_token[0..16],
            &captcha_token[16..32],
            &password_data,
        )?)?;
        Ok(password)
    }
}

const WIDTH: u32 = 220;
const HEIGHT: u32 = 120;

pub async fn gen_captcha(app: Data<Arc<ShareData>>) -> actix_web::Result<impl Responder> {
    let token = uuid::Uuid::new_v4().to_string().replace('-', "");
    let captcha_cookie = Cookie::build("ratch_captcha_token", token.as_str())
        .path("/")
        .http_only(true)
        .finish();
    let captcha_header = ("Captcha-Token", token.as_str());

    // 如果验证码功能被禁用，data 为 null
    if !app.app_config.console_captcha_enable {
        return Ok(HttpResponse::Ok()
            .cookie(captcha_cookie)
            .insert_header(captcha_header)
            .json(ApiResult::<String>::success(None)));
    }

    //let obj = gen(Difficulty::Easy);
    let mut obj = Captcha::new();
    obj.add_chars(4)
        .apply_filter(Noise::new(0.1))
        .apply_filter(Grid::new(8, 8))
        .view(WIDTH, HEIGHT);

    let code: String = obj.chars().iter().collect::<String>().to_uppercase();
    let code = Arc::new(code);

    let img = obj.as_base64().unwrap_or_default();
    //log::info!("gen_captcha code:{}", &code);
    let now = now_second_i32();
    let set_info = SetInfo {
        key: CacheKey::new(CacheType::String, Arc::new(format!("Captcha_{}", &token))),
        value: CacheValue::String(code),
        ttl: 300,
        now,
        nx: false,
        xx: false,
    };
    let cache_req = CacheManagerRaftReq::Set(set_info);
    app.raft_request_route
        .request(ClientRequest::CacheReq { req: cache_req })
        .await
        .ok();
    Ok(HttpResponse::Ok()
        .cookie(captcha_cookie)
        .insert_header(captcha_header)
        .json(ApiResult::success(Some(img))))
}

pub async fn logout(
    request: HttpRequest,
    app: Data<Arc<ShareData>>,
) -> actix_web::Result<impl Responder> {
    let token = if let Some(ck) = request.cookie(CONSOLE_TOKEN_COOKIE_KEY) {
        ck.value().to_owned()
    } else if let Some(v) = request.headers().get("Token") {
        v.to_str().unwrap_or_default().to_owned()
    } else {
        "".to_owned()
    };
    let token = Arc::new(token);
    let cache_req = CacheManagerRaftReq::Remove(CacheKey::new(CacheType::UserSession, token));
    app.raft_request_route
        .request(ClientRequest::CacheReq { req: cache_req })
        .await
        .ok();
    return Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build(CONSOLE_TOKEN_COOKIE_KEY, "")
                .path("/")
                .http_only(true)
                .finish(),
        )
        .json(ApiResult::success(Some(true))));
}
