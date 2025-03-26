use crate::task::model::request_model::JobRunParam;
use crate::task::model::task::JobTaskInfo;
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskRequestResult>")]
pub enum TaskRequestCmd {
    RunTask(Arc<String>, JobRunParam, JobTaskInfo),
    RunBroadcastTask(Vec<Arc<String>>, JobRunParam),
}

pub enum TaskRequestResult {
    None,
}
