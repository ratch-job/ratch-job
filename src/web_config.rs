use crate::common::app_config::AppConfig;
use crate::common::get_app_version;
use crate::console::v1::console_api_v1;
use crate::openapi::openapi_config;
use crate::openapi::xxljob::xxl_api_config;
use actix_web::web::ServiceConfig;
use actix_web::{web, HttpResponse, Responder};
use mime_guess::from_path;
use ratchjob_web_dist_wrap::get_embedded_file;
use std::sync::Arc;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match get_embedded_file(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

fn handle_embedded_file_with_cache(path: &str) -> HttpResponse {
    match get_embedded_file(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .insert_header(("Cache-Control", "max-age=604800, public"))
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

pub(crate) async fn index() -> impl Responder {
    handle_embedded_file("index.html")
}

#[actix_web::get("/server.svg")]
pub(crate) async fn icon() -> impl Responder {
    handle_embedded_file_with_cache("server.svg")
}

#[actix_web::get("/ratchjob/server.svg")]
pub(crate) async fn console_icon() -> impl Responder {
    handle_embedded_file_with_cache("ratchjob/server.svg")
}

#[actix_web::get("/assets/{_:.*}")]
pub(crate) async fn assets(path: web::Path<String>) -> impl Responder {
    let file = format!("assets/{}", path.as_ref());
    handle_embedded_file_with_cache(&file)
}

#[actix_web::get("/ratchjob/assets/{_:.*}")]
pub(crate) async fn console_assets(path: web::Path<String>) -> impl Responder {
    let file = format!("ratchjob/assets/{}", path.as_ref());
    handle_embedded_file_with_cache(&file)
}

async fn disable_no_auth_console_index() -> impl Responder {
    let body = "<!DOCTYPE html>
<html lang='en'>
  <head>
    <meta charset='UTF-8' />
    <meta name='viewport' content='width=device-width, initial-scale=1.0' />
    <title>RATCH-JOB</title>
  </head>
  <body>
    <p>请使用控制台: http://localhost:8845/ratchjob/ </p>
  </body>
</html>";
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn about_info() -> impl Responder {
    format!("ratch-job version:{}", get_app_version())
}

pub fn app_config(app_config: Arc<AppConfig>) -> impl FnOnce(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        openapi_config(config);
        xxl_api_config(config, &app_config);
    }
}

pub fn console_config(config: &mut ServiceConfig) {
    console_page_config(config);
    console_api_v1(config);
}
pub fn console_page_config(config: &mut ServiceConfig) {
    config
        .service(web::resource("/").route(web::get().to(index)))
        .service(icon)
        .service(assets)
        .service(web::resource("/index.html").route(web::get().to(index)))
        .service(web::resource("/404").route(web::get().to(index)))
        .service(web::resource("/nopermission").route(web::get().to(index)))
        .service(web::resource("/manage/{_:.*}").route(web::get().to(index)))
        .service(web::resource("/p/{_:.*}").route(web::get().to(index)))
        //new console path
        .service(web::resource("/ratchjob").route(web::get().to(index)))
        .service(web::resource("/ratchjob/").route(web::get().to(index)))
        .service(console_icon)
        .service(console_assets)
        .service(web::resource("/ratchjob/index.html").route(web::get().to(index)))
        .service(web::resource("/ratchjob/404").route(web::get().to(index)))
        .service(web::resource("/ratchjob/nopermission").route(web::get().to(index)))
        .service(web::resource("/ratchjob/manage/{_:.*}").route(web::get().to(index)))
        .service(web::resource("/ratchjob/p/{_:.*}").route(web::get().to(index)));
}
