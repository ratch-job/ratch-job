use crate::common::share_data::ShareData;
use crate::openapi::xxljob::model::server_request::{CallbackParam, RegistryParam};
use crate::openapi::xxljob::model::xxl_api_empty_success;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub(crate) async fn register(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<RegistryParam>,
) -> impl Responder {
    HttpResponse::Ok().json(xxl_api_empty_success())
}

pub(crate) async fn unregister(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<RegistryParam>,
) -> impl Responder {
    HttpResponse::Ok().json(xxl_api_empty_success())
}

pub(crate) async fn callback(
    share_data: Data<Arc<ShareData>>,
    web::Json(param): web::Json<CallbackParam>,
) -> impl Responder {
    HttpResponse::Ok().json(xxl_api_empty_success())
}
