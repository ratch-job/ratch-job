use crate::job::model::job::JobInfo;
use actix::Message;
use std::sync::Arc;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<ScheduleManagerResult>")]
pub enum ScheduleManagerReq {
    UpdateJob(Arc<JobInfo>),
    RemoveJob(u64),
}

pub enum ScheduleManagerResult {
    None,
}
