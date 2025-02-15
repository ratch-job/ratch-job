use crate::common::constant::DEFAULT_XXL_NAMESPACE;
use crate::common::model::ApiResult;
use crate::common::share_data::ShareData;
use crate::console::model::namespace_model::NamespaceInfo;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub async fn query_namespace_list(
    _req: HttpRequest,
    _app_data: web::Data<Arc<ShareData>>,
) -> impl Responder {
    //前期先只返回默认命名空间
    let list = vec![NamespaceInfo {
        namespace_id: Some(DEFAULT_XXL_NAMESPACE.clone()),
        namespace_name: Some(DEFAULT_XXL_NAMESPACE.as_str().to_string()),
        r#type: None,
    }];
    HttpResponse::Ok().json(ApiResult::success(Some(list)))
}
