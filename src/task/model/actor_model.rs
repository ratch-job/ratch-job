use crate::app::model::{AppInstanceKey, AppKey};
use crate::common::constant::EMPTY_ARC_STR;
use crate::job::model::job::{JobInfo, JobTaskLogQueryParam};
use crate::task::model::task::{JobTaskInfo, TaskCallBackParam};
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum TriggerSource {
    System,
    User(Arc<String>),
}

#[derive(Debug, Clone)]
pub struct TriggerItem {
    pub trigger_time: u32,
    pub job_info: Arc<JobInfo>,
    pub fix_addr: Arc<String>,
    pub trigger_source: TriggerSource,
}

impl TriggerItem {
    pub fn new(trigger_time: u32, job_info: Arc<JobInfo>) -> Self {
        TriggerItem {
            trigger_time,
            job_info,
            fix_addr: EMPTY_ARC_STR.clone(),
            trigger_source: TriggerSource::System,
        }
    }

    pub fn new_with_user(
        trigger_time: u32,
        job_info: Arc<JobInfo>,
        fix_addr: Arc<String>,
        user_name: Arc<String>,
    ) -> Self {
        TriggerItem {
            trigger_time,
            job_info,
            fix_addr,
            trigger_source: TriggerSource::User(user_name),
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskManagerResult>")]
pub enum TaskManagerReq {
    AddAppInstance(AppKey, Arc<String>),
    AddAppInstances(Vec<AppInstanceKey>),
    RemoveAppInstance(AppKey, Arc<String>),
    RemoveAppInstances(Vec<AppInstanceKey>),
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
