use crate::app::model::AppKey;
use crate::job::model::job::{JobInfo, JobTaskLogQueryParam};
use crate::task::model::task::{JobTaskInfo, TaskCallBackParam};
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TriggerItem {
    pub trigger_time: u32,
    pub job_info: Arc<JobInfo>,
}

impl TriggerItem {
    pub fn new(trigger_time: u32, job_info: Arc<JobInfo>) -> Self {
        TriggerItem {
            trigger_time,
            job_info,
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskManagerResult>")]
pub enum TaskManagerReq {
    AddAppInstance(AppKey, Arc<String>),
    RemoveAppInstance(AppKey, Arc<String>),
    TriggerTask(u32, Arc<JobInfo>),
    TriggerTaskList(Vec<TriggerItem>),
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
