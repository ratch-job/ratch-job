use crate::app::model::{AppInstanceKey, AppKey};
use crate::common::constant::{EMPTY_ARC_STR, TRIGGER_FROM_SYSTEM};
use crate::job::model::job::{JobInfo, JobTaskLogQueryParam};
use crate::task::model::task::{JobTaskInfo, TaskCallBackParam};
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum TriggerSourceType {
    System,
    User(Arc<String>),
}

impl TriggerSourceType {
    pub fn get_source_from(&self) -> Arc<String> {
        match self {
            TriggerSourceType::System => TRIGGER_FROM_SYSTEM.clone(),
            TriggerSourceType::User(user) => user.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriggerSourceInfo {
    pub fix_addr: Arc<String>,
    pub source_type: TriggerSourceType,
}

impl Default for TriggerSourceInfo {
    fn default() -> Self {
        TriggerSourceInfo {
            fix_addr: EMPTY_ARC_STR.clone(),
            source_type: TriggerSourceType::System,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriggerItem {
    pub trigger_time: u32,
    pub job_info: Arc<JobInfo>,
    pub trigger_source: TriggerSourceInfo,
}

impl TriggerItem {
    pub fn new(trigger_time: u32, job_info: Arc<JobInfo>) -> Self {
        TriggerItem {
            trigger_time,
            job_info,
            trigger_source: TriggerSourceInfo {
                fix_addr: EMPTY_ARC_STR.clone(),
                source_type: TriggerSourceType::System,
            },
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
            trigger_source: TriggerSourceInfo {
                fix_addr,
                source_type: TriggerSourceType::User(user_name),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryTaskItem {
    pub trigger_time: u32,
    pub task_info: JobTaskInfo,
    pub job_info: Option<Arc<JobInfo>>,
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<TaskManagerResult>")]
pub enum TaskManagerReq {
    AddAppInstance(AppKey, Arc<String>),
    AddAppInstances(Vec<AppInstanceKey>),
    RemoveAppInstance(AppKey, Arc<String>),
    RemoveAppInstances(Vec<AppInstanceKey>),
    TriggerTaskList(Vec<TriggerItem>),
    RetryTaskList(Vec<RetryTaskItem>),
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
