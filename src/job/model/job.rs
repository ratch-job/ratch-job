use crate::job::model::enum_type::{
    ExecutorBlockStrategy, JobRunMode, PastDueStrategy, RouterStrategy, ScheduleType,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub id: u64,
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub description: Arc<String>,
    pub schedule_type: ScheduleType,
    pub cron_value: Arc<String>,
    pub delay_second: u32,
    pub interval_second: u32,
    pub run_mode: JobRunMode,
    pub handle_name: Arc<String>,
    pub trigger_param: Arc<String>,
    pub router_strategy: RouterStrategy,
    pub past_due_strategy: PastDueStrategy,
    pub blocking_strategy: ExecutorBlockStrategy,
    pub timeout_second: u32,
    pub try_times: u32,
    pub histories: Vec<JobHistoryInfo>,
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub register_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobHistoryInfo {
    pub version_id: u64,
    pub schedule_type: ScheduleType,
    pub cron_value: Arc<String>,
    pub delay_second: u32,
    pub interval_second: u32,
    pub trigger_mode: Arc<String>,
    pub handle_name: Arc<String>,
    pub trigger_param: Arc<String>,
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobKey {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub handle_name: Arc<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobParam {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub handle_name: Arc<String>,
    pub description: Option<Arc<String>>,
    pub schedule_type: Option<ScheduleType>,
    pub cron_value: Option<Arc<String>>,
    pub delay_second: Option<u32>,
    pub interval_second: Option<u32>,
    pub run_mode: Option<JobRunMode>,
    pub trigger_param: Option<Arc<String>>,
    pub router_strategy: Option<RouterStrategy>,
    pub past_due_strategy: Option<PastDueStrategy>,
    pub blocking_strategy: Option<ExecutorBlockStrategy>,
    pub timeout_second: Option<u32>,
    pub try_times: Option<u32>,
}
