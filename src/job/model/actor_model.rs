use crate::job::model::job::{JobInfo, JobKey, JobParam};
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<JobManagerResult>")]
pub enum JobManagerReq {
    AddJob(JobParam),
    UpdateJob(JobParam),
    Remove(u64),
    GetJob(u64),
    GetJobByKey(JobKey),
}

#[derive(Debug, Clone)]
pub enum JobManagerResult {
    JobInfo(Arc<JobInfo>),
    None,
}
