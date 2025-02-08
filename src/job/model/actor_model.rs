use crate::job::model::job::{JobKey, JobParam};
use actix::Message;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<JobManagerResult>")]
pub enum JobManagerReq {
    UpdateJob(JobParam),
    Remove(JobKey),
}

#[derive(Debug, Clone)]
pub enum JobManagerResult {
    None,
}
