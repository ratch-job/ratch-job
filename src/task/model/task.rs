use crate::common::constant::EMPTY_ARC_STR;
use crate::job::model::job::JobInfo;
use crate::task::model::enum_type::TaskStatusType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobTaskInfo {
    pub task_id: u64,
    pub job_id: u64,
    pub trigger_time: u32,
    pub instance_addr: Arc<String>,
    pub trigger_message: Arc<String>,
    pub status: TaskStatusType,
    pub finish_time: u32,
    pub callback_message: Arc<String>,
}

impl JobTaskInfo {
    pub fn from_job(trigger_time: u32, job: &Arc<JobInfo>) -> Self {
        JobTaskInfo {
            task_id: 0,
            job_id: job.id,
            trigger_time,
            instance_addr: EMPTY_ARC_STR.clone(),
            trigger_message: EMPTY_ARC_STR.clone(),
            status: TaskStatusType::Init,
            finish_time: 0,
            callback_message: EMPTY_ARC_STR.clone(),
        }
    }
}
