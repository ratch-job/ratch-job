use crate::task::model::request_model::JobRunParam;
use crate::task::model::task::JobTaskInfo;
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskRequestResult>")]
pub enum TaskRequestCmd {
    RunTask(Arc<String>, JobRunParam, JobTaskInfo),
    RunBroadcastTask(Arc<Vec<Arc<String>>>, JobRunParam),
}

impl TaskRequestCmd {
    pub fn get_task(self) -> Option<JobTaskInfo> {
        match self {
            TaskRequestCmd::RunTask(_, _, task) => Some(task),
            TaskRequestCmd::RunBroadcastTask(_, _) => None,
        }
    }
}

pub enum TaskRequestResult {
    None,
    RunningCount(usize),
}
