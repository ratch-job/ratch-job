use crate::job::job_index::JobQueryParam;
use crate::job::model::job::{JobInfo, JobInfoDto, JobParam, JobTaskLogQueryParam};
use crate::task::model::task::JobTaskInfo;
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<JobManagerResult>")]
pub enum JobManagerReq {
    AddJob(JobParam),
    UpdateJob(JobParam),
    UpdateTask(Arc<JobTaskInfo>),
    Remove(u64),
    GetJob(u64),
    QueryJob(JobQueryParam),
    QueryJobTaskLog(JobTaskLogQueryParam),
}

#[derive(Debug, Clone)]
pub enum JobManagerResult {
    JobInfo(Option<Arc<JobInfo>>),
    JobPageInfo(usize, Vec<JobInfoDto>),
    JobTaskLogPageInfo(usize, Vec<Arc<JobTaskInfo>>),
    None,
}
