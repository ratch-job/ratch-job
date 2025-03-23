use crate::common::constant::EMPTY_ARC_STR;
use crate::common::pb::data_object::{JobTaskDo, TaskTryLogDo};
use crate::job::model::job::JobInfo;
use crate::task::model::actor_model::TriggerSourceInfo;
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
    pub retry_interval: u32,
    pub retry_count: u32,
    pub timeout_second: u32,
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
            retry_interval: job.retry_interval,
            retry_count: 0,
            timeout_second: job.timeout_second,
        }
    }

    pub fn can_retry(&self) -> bool {
        self.try_times > self.retry_count
    }

    pub fn push_next_try(&mut self) {
        self.retry_count += 1;
        self.try_logs.push(TaskTryLog {
            execution_time: self.execution_time,
            addr: self.instance_addr.clone(),
        });
        self.execution_time = 0;
        self.instance_addr = EMPTY_ARC_STR.clone();
        self.status = TaskStatusType::Running;
    }

    pub fn get_retry_interval(&self) -> u32 {
        if self.retry_interval > 0 {
            self.retry_interval
        } else {
            //默认间隔10秒
            10
        }
    }

    pub fn get_timeout_second(&self, default_value: u32) -> u32 {
        if self.timeout_second > 0 {
            self.timeout_second
        } else {
            default_value
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
            retry_interval: self.retry_interval,
            retry_count: self.retry_count,
            timeout_second: self.timeout_second,
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
            retry_interval: task_do.retry_interval,
            retry_count: task_do.retry_count,
            timeout_second: task_do.timeout_second,
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

#[derive(Debug, Clone, Default)]
pub struct UpdateTaskMetricsInfo {
    pub success_count: u64,
    pub fail_count: u64,
}

impl UpdateTaskMetricsInfo {
    pub fn add(&mut self, task_info: &UpdateTaskMetricsInfo) {
        self.success_count += task_info.success_count;
        self.fail_count += task_info.fail_count;
    }
}
