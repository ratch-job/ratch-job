use crate::common::get_app_version;
use crate::common::model::ApiResult;
use crate::common::share_data::ShareData;
use crate::console::model::user_model::UserPermissions;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub async fn get_user_web_resources(
    _req: HttpRequest,
    _app_data: web::Data<Arc<ShareData>>,
) -> impl Responder {
    let data = UserPermissions {
        from: "OLD_CONSOLE",
        version: get_app_version(),
        username: None,
        resources: vec![],
    };
    HttpResponse::Ok().json(ApiResult::success(Some(data)))
}
