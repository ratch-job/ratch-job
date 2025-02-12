use crate::app::model::AppKey;
use crate::job::model::job::JobInfo;
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskManagerResult>")]
pub enum TaskManagerReq {
    AddAppInstance(AppKey, Arc<String>),
    RemoveAppInstance(AppKey, Arc<String>),
    TriggerTask(u32, Arc<JobInfo>),
}

pub enum TaskManagerResult {
    None,
}
