use crate::app::model::AppKey;
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskManagerResult>")]
pub enum TaskManagerReq {
    InitAppInstance(AppKey, Vec<Arc<String>>),
    AddAppInstance(AppKey, Vec<Arc<String>>),
    RemoveAppInstance(AppKey, Vec<Arc<String>>),
}

pub enum TaskManagerResult {
    None,
}
