use crate::app::model::AppKey;
use crate::common::constant::EMPTY_ARC_STR;
use crate::common::pb::data_object::JobTaskDo;
use crate::job::model::job::JobInfo;
use crate::task::model::actor_model::TriggerItem;
use crate::task::model::app_instance::InstanceAddrSelectResult;
use crate::task::model::enum_type::TaskStatusType;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
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
}
