use crate::common::constant::EMPTY_ARC_STR;
use crate::common::cron_utils::CronUtil;
use crate::job::model::enum_type::{
    ExecutorBlockStrategy, JobRunMode, PastDueStrategy, RouterStrategy, ScheduleType,
};
use crate::task::model::task::JobTaskInfo;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub register_time: u64,
}

impl JobInfo {
    pub fn update_param(&mut self, job_param: JobParam) {
        if let Some(description) = job_param.description {
            self.description = description;
        }
        if let Some(schedule_type) = job_param.schedule_type {
            self.schedule_type = schedule_type;
        }
        if let Some(cron_value) = job_param.cron_value {
            self.cron_value = cron_value;
        }
        if let Some(delay_second) = job_param.delay_second {
            self.delay_second = delay_second;
        }
        if let Some(interval_second) = job_param.interval_second {
            self.interval_second = interval_second;
        }
        if let Some(run_mode) = job_param.run_mode {
            self.run_mode = run_mode;
        }
        if let Some(handle_name) = job_param.handle_name {
            self.handle_name = handle_name;
        }
        if let Some(trigger_param) = job_param.trigger_param {
            self.trigger_param = trigger_param;
        }
        if let Some(router_strategy) = job_param.router_strategy {
            self.router_strategy = router_strategy;
        }
        if let Some(past_due_strategy) = job_param.past_due_strategy {
            self.past_due_strategy = past_due_strategy;
        }
        if let Some(blocking_strategy) = job_param.blocking_strategy {
            self.blocking_strategy = blocking_strategy;
        }
        if let Some(timeout_second) = job_param.timeout_second {
            self.timeout_second = timeout_second;
        }
        if let Some(try_times) = job_param.try_times {
            self.try_times = try_times;
        }
    }

    pub fn get_key(&self) -> JobKey {
        JobKey::new(
            self.app_name.clone(),
            self.namespace.clone(),
            self.handle_name.clone(),
        )
    }

    pub fn check_valid(&self) -> anyhow::Result<()> {
        if self.id == 0 {
            Err(anyhow::anyhow!("id is empty!"))
        } else if self.namespace.is_empty() || self.app_name.is_empty() {
            Err(anyhow::anyhow!("namespace or app_name is empty!"))
        } else if self.run_mode == JobRunMode::Bean && self.handle_name.is_empty() {
            Err(anyhow::anyhow!("bean handle_name is invalid!"))
        } else if self.schedule_type == ScheduleType::Cron
            && !CronUtil::check_cron_valid(&self.cron_value)
        {
            Err(anyhow::anyhow!("cron_value is invalid!"))
        } else {
            Ok(())
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.id == 0 || self.namespace.is_empty() || self.app_name.is_empty() {
            false
        } else if self.run_mode == JobRunMode::Bean && self.handle_name.is_empty() {
            false
        } else if self.schedule_type == ScheduleType::Cron
            && !CronUtil::check_cron_valid(&self.cron_value)
        {
            false
        } else {
            true
        }
    }
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

pub struct JobWrap {
    pub job: Arc<JobInfo>,
    pub histories: Vec<Arc<JobHistoryInfo>>,
    pub task_log_map: BTreeMap<u64, Arc<JobTaskInfo>>,
}

impl JobWrap {
    pub fn new(job: Arc<JobInfo>) -> Self {
        Self {
            job,
            histories: vec![],
            task_log_map: BTreeMap::new(),
        }
    }

    pub fn update_task_log(&mut self, new_task_log: Arc<JobTaskInfo>, limit_count: usize) {
        if let Some(task_log) = self.task_log_map.get_mut(&new_task_log.task_id) {
            *task_log = new_task_log;
        } else {
            self.task_log_map.insert(new_task_log.task_id, new_task_log);
            if self.task_log_map.len() > limit_count {
                self.task_log_map.pop_first();
            }
        }
    }
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobKey {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub handle_name: Arc<String>,
}

impl JobKey {
    pub fn new(app_name: Arc<String>, namespace: Arc<String>, handle_name: Arc<String>) -> Self {
        JobKey {
            app_name,
            namespace,
            handle_name,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobParam {
    pub id: Option<u64>,
    pub app_name: Option<Arc<String>>,
    pub namespace: Option<Arc<String>>,
    pub description: Option<Arc<String>>,
    pub schedule_type: Option<ScheduleType>,
    pub cron_value: Option<Arc<String>>,
    pub delay_second: Option<u32>,
    pub interval_second: Option<u32>,
    pub run_mode: Option<JobRunMode>,
    pub handle_name: Option<Arc<String>>,
    pub trigger_param: Option<Arc<String>>,
    pub router_strategy: Option<RouterStrategy>,
    pub past_due_strategy: Option<PastDueStrategy>,
    pub blocking_strategy: Option<ExecutorBlockStrategy>,
    pub timeout_second: Option<u32>,
    pub try_times: Option<u32>,
}

impl From<JobParam> for JobInfo {
    fn from(job_param: JobParam) -> Self {
        JobInfo {
            id: job_param.id.unwrap_or_default(),
            app_name: job_param.app_name.unwrap_or_default(),
            namespace: job_param.namespace.unwrap_or_default(),
            handle_name: job_param.handle_name.unwrap_or(EMPTY_ARC_STR.clone()),
            description: job_param.description.unwrap_or(EMPTY_ARC_STR.clone()),
            schedule_type: job_param.schedule_type.unwrap_or(ScheduleType::None),
            cron_value: job_param.cron_value.unwrap_or(EMPTY_ARC_STR.clone()),
            delay_second: job_param.delay_second.unwrap_or_default(),
            interval_second: job_param.interval_second.unwrap_or_default(),
            run_mode: job_param.run_mode.unwrap_or(JobRunMode::Bean),
            trigger_param: job_param.trigger_param.unwrap_or(EMPTY_ARC_STR.clone()),
            router_strategy: job_param
                .router_strategy
                .unwrap_or(RouterStrategy::RoundRobin),
            past_due_strategy: job_param
                .past_due_strategy
                .unwrap_or(PastDueStrategy::Default),
            blocking_strategy: job_param
                .blocking_strategy
                .unwrap_or(ExecutorBlockStrategy::SerialExecution),
            timeout_second: job_param.timeout_second.unwrap_or_default(),
            try_times: job_param.try_times.unwrap_or_default(),
            version_id: 0,
            last_modified_millis: 0,
            register_time: 0,
        }
    }
}
