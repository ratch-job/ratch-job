use crate::common::share_data::ShareData;
use actix_web::web::{Data, Json};
use actix_web::Responder;
use std::sync::Arc;

pub async fn metrics(app: Data<Arc<ShareData>>) -> actix_web::Result<impl Responder> {
    let metrics = app.raft.metrics().borrow().clone();
    Ok(Json(metrics))
}
