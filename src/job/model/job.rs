use crate::app::model::AppKey;
use crate::common::constant::EMPTY_ARC_STR;
use crate::common::cron_utils::CronUtil;
use crate::common::pb::data_object::JobDo;
use crate::job::model::enum_type::{
    ExecutorBlockStrategy, JobRunMode, PastDueStrategy, RouterStrategy, ScheduleType,
};
use crate::task::model::task::JobTaskInfo;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub id: u64,
    pub enable: bool,
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
    pub create_time: u64,
}

impl JobInfo {
    pub fn update_param(&mut self, job_param: JobParam) {
        if let Some(description) = job_param.description {
            self.description = description;
        }
        if let Some(enable) = job_param.enable {
            self.enable = enable;
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
        if let Some(update_time) = job_param.update_time {
            self.create_time = update_time;
        }
        self.version_id += 1;
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

    pub fn build_app_key(&self) -> AppKey {
        AppKey {
            namespace: self.namespace.clone(),
            app_name: self.app_name.clone(),
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

    pub fn to_do(&self) -> JobDo {
        JobDo {
            id: self.id,
            enable: self.enable,
            app_name: Cow::Borrowed(&self.app_name),
            namespace: Cow::Borrowed(&self.namespace),
            description: Cow::Borrowed(&self.description),
            schedule_type: Cow::Borrowed(self.schedule_type.to_str()),
            cron_value: Cow::Borrowed(&self.cron_value),
            delay_second: self.delay_second,
            interval_second: self.interval_second,
            run_mode: Cow::Borrowed(self.run_mode.to_str()),
            handle_name: Cow::Borrowed(&self.handle_name),
            trigger_param: Cow::Borrowed(&self.trigger_param),
            router_strategy: Cow::Borrowed(self.router_strategy.to_str()),
            past_due_strategy: Cow::Borrowed(self.past_due_strategy.to_str()),
            blocking_strategy: Cow::Borrowed(self.blocking_strategy.to_str()),
            timeout_second: self.timeout_second,
            try_times: self.try_times,
            version_id: self.version_id,
            last_modified_millis: self.last_modified_millis,
            create_time: self.create_time,
        }
    }
}

impl<'a> From<JobDo<'a>> for JobInfo {
    fn from(job_do: JobDo<'a>) -> Self {
        JobInfo {
            id: job_do.id,
            enable: job_do.enable,
            app_name: Arc::new(job_do.app_name.to_string()),
            namespace: Arc::new(job_do.namespace.to_string()),
            description: Arc::new(job_do.description.to_string()),
            schedule_type: ScheduleType::from_str(&job_do.schedule_type),
            cron_value: Arc::new(job_do.cron_value.to_string()),
            delay_second: job_do.delay_second,
            interval_second: job_do.interval_second,
            run_mode: JobRunMode::from_str(&job_do.run_mode).unwrap_or(JobRunMode::Bean),
            handle_name: Arc::new(job_do.handle_name.to_string()),
            trigger_param: Arc::new(job_do.trigger_param.to_string()),
            router_strategy: RouterStrategy::from_str(&job_do.router_strategy)
                .unwrap_or(RouterStrategy::RoundRobin),
            past_due_strategy: PastDueStrategy::from_str(&job_do.past_due_strategy),
            blocking_strategy: ExecutorBlockStrategy::from_str(&job_do.blocking_strategy),
            timeout_second: job_do.timeout_second,
            try_times: job_do.try_times,
            version_id: job_do.version_id,
            last_modified_millis: job_do.last_modified_millis,
            create_time: job_do.create_time,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobParam {
    pub id: Option<u64>,
    pub enable: Option<bool>,
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
    pub update_time: Option<u64>,
}

impl From<JobParam> for JobInfo {
    fn from(job_param: JobParam) -> Self {
        JobInfo {
            id: job_param.id.unwrap_or_default(),
            enable: job_param.enable.unwrap_or(true),
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
            last_modified_millis: job_param.update_time.unwrap_or(0),
            create_time: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobTaskLogQueryParam {
    pub job_id: u64,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfoDto {
    pub id: u64,
    pub enable: bool,
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub description: Arc<String>,
    pub schedule_type: String,
    pub cron_value: Arc<String>,
    pub delay_second: u32,
    pub interval_second: u32,
    pub run_mode: String,
    pub handle_name: Arc<String>,
    pub trigger_param: Arc<String>,
    pub router_strategy: String,
    pub past_due_strategy: String,
    pub blocking_strategy: String,
    pub timeout_second: u32,
    pub try_times: u32,
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub register_time: u64,
}

impl JobInfoDto {
    pub fn new_from(job_info: &JobInfo) -> Self {
        JobInfoDto {
            id: job_info.id,
            enable: job_info.enable,
            app_name: job_info.app_name.clone(),
            namespace: job_info.namespace.clone(),
            description: job_info.description.clone(),
            schedule_type: job_info.schedule_type.to_str().to_owned(),
            cron_value: job_info.cron_value.clone(),
            delay_second: job_info.delay_second,
            interval_second: job_info.interval_second,
            run_mode: job_info.run_mode.to_str().to_owned(),
            handle_name: job_info.handle_name.clone(),
            trigger_param: job_info.trigger_param.clone(),
            router_strategy: job_info.router_strategy.to_str().to_owned(),
            past_due_strategy: job_info.past_due_strategy.to_str().to_owned(),
            blocking_strategy: job_info.blocking_strategy.to_str().to_owned(),
            timeout_second: job_info.timeout_second,
            try_times: job_info.try_times,
            version_id: job_info.version_id,
            last_modified_millis: job_info.last_modified_millis,
            register_time: job_info.create_time,
        }
    }
}
