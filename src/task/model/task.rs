use crate::app::model::AppKey;
use crate::common::constant::EMPTY_ARC_STR;
use crate::common::pb::data_object::{JobTaskDo, TaskTryLogDo};
use crate::job::model::job::JobInfo;
use crate::task::model::actor_model::{TriggerItem, TriggerSourceInfo};
use crate::task::model::app_instance::InstanceAddrSelectResult;
use crate::task::model::enum_type::TaskStatusType;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTryLog {
    pub execution_time: u32,
    pub addr: Arc<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobTaskInfo {
    pub task_id: u64,
    pub job_id: u64,
    pub trigger_time: u32,
    pub instance_addr: Arc<String>,
    pub trigger_message: Arc<String>,
    pub status: TaskStatusType,
    pub finish_time: u32,
    pub callback_message: Arc<String>,
    pub execution_time: u32,
    pub trigger_from: Arc<String>,
    pub try_times: u32,
    pub try_logs: Vec<TaskTryLog>,
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
            execution_time: 0,
            trigger_from: EMPTY_ARC_STR.clone(),
            try_times: job.try_times,
            try_logs: vec![],
        }
    }

    pub fn to_do(&self) -> JobTaskDo {
        JobTaskDo {
            task_id: self.task_id,
            job_id: self.job_id,
            trigger_time: self.trigger_time,
            instance_addr: Cow::Borrowed(&self.instance_addr),
            trigger_message: Cow::Borrowed(&self.trigger_message),
            status: Cow::Borrowed(self.status.to_str()),
            finish_time: self.finish_time,
            callback_message: Cow::Borrowed(&self.callback_message),
            execution_time: self.execution_time,
            trigger_from: Cow::Borrowed(&self.trigger_from),
            try_times: self.try_times,
            try_logs: self
                .try_logs
                .iter()
                .map(|log| TaskTryLogDo {
                    execution_time: log.execution_time,
                    addr: Cow::Borrowed(&log.addr),
                })
                .collect(),
        }
    }
}

impl<'a> From<JobTaskDo<'a>> for JobTaskInfo {
    fn from(task_do: JobTaskDo<'a>) -> Self {
        JobTaskInfo {
            task_id: task_do.task_id,
            job_id: task_do.job_id,
            trigger_time: task_do.trigger_time,
            instance_addr: Arc::new(task_do.instance_addr.to_string()),
            trigger_message: Arc::new(task_do.trigger_message.to_string()),
            status: TaskStatusType::from_str(&task_do.status),
            finish_time: task_do.finish_time,
            callback_message: Arc::new(task_do.callback_message.to_string()),
            execution_time: task_do.execution_time,
            trigger_from: Arc::new(task_do.trigger_from.to_string()),
            try_times: task_do.try_times,
            try_logs: task_do
                .try_logs
                .iter()
                .map(|log| TaskTryLog {
                    execution_time: log.execution_time,
                    addr: Arc::new(log.addr.to_string()),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCallBackParam {
    pub task_id: u64,
    pub task_date_time: i64,
    pub success: bool,
    pub handle_msg: Option<String>,
}

pub struct TaskWrap {
    pub task: JobTaskInfo,
    pub job_info: Arc<JobInfo>,
    pub select_result: InstanceAddrSelectResult,
    pub app_addrs: Vec<Arc<String>>,
    pub trigger_source: TriggerSourceInfo,
}
