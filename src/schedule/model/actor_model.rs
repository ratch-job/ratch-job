use crate::job::model::job::{JobInfo, JobTaskLogQueryParam};
use crate::schedule::model::DelayFinishTasks;
use crate::task::model::task::{JobTaskInfo, TaskCallBackParam};
use actix::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<ScheduleManagerResult>")]
pub enum ScheduleManagerReq {
    UpdateJob(Arc<JobInfo>),
    RemoveJob(u64),
    UpdateTask(Arc<JobTaskInfo>),
    DelayFinishTasks(DelayFinishTasks),
    UpdateTaskList(Vec<Arc<JobTaskInfo>>),
    QueryJobTaskLog(JobTaskLogQueryParam),
}

pub enum ScheduleManagerResult {
    JobTaskLogPageInfo(usize, Vec<Arc<JobTaskInfo>>),
    None,
}

#[derive(Clone, Debug, Message, Serialize, Deserialize)]
#[rtype(result = "anyhow::Result<ScheduleManagerRaftResult>")]
pub enum ScheduleManagerRaftReq {
    TaskCallBacks(Vec<TaskCallBackParam>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScheduleManagerRaftResult {
    None,
}
