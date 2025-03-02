use crate::app::model::AppKey;
use crate::job::model::job::{JobInfo, JobTaskLogQueryParam};
use crate::task::model::task::{JobTaskInfo, TaskCallBackParam};
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskManagerResult>")]
pub enum TaskManagerReq {
    AddAppInstance(AppKey, Arc<String>),
    RemoveAppInstance(AppKey, Arc<String>),
    TriggerTask(u32, Arc<JobInfo>),
    TaskCallBacks(Vec<TaskCallBackParam>),
}

pub enum TaskManagerResult {
    None,
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskHistoryManagerResult>")]
pub enum TaskHistoryManagerReq {
    UpdateTask(Arc<JobTaskInfo>),
    QueryJobTaskLog(JobTaskLogQueryParam),
}

pub enum TaskHistoryManagerResult {
    JobTaskLogPageInfo(usize, Vec<Arc<JobTaskInfo>>),
    None,
}
