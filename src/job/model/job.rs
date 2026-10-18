use crate::app::model::AppKey;
use crate::common::constant::EMPTY_ARC_STR;
use crate::common::cron_utils::CronUtil;
use crate::common::pb::data_object::JobDo;
use crate::common::string_utils::StringUtils;
use crate::job::model::enum_type::{
    ExecutorBlockStrategy, JobRunMode, PastDueStrategy, RouterStrategy, ScheduleType,
};
use crate::task::model::enum_type::TaskStatusType;
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
    pub key: Arc<String>,
    pub description: Arc<String>,
    pub schedule_type: ScheduleType,
    pub cron_value: Arc<String>,
    pub delay_second: u32,
    pub interval_second: u32,
    pub run_mode: JobRunMode,
    pub handle_name: Arc<String>,
    pub trigger_param: Arc<String>,
    #[serde(default)]
    pub glue_source: Arc<String>,
    pub router_strategy: RouterStrategy,
    pub past_due_strategy: PastDueStrategy,
    pub blocking_strategy: ExecutorBlockStrategy,
    pub timeout_second: u32,
    pub try_times: u32,
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub create_time: u64,
    pub retry_interval: u32,
}

impl JobInfo {
    pub fn update_param(&mut self, job_param: JobParam) {
        // Capture before applying params: only already-GLUE legacy jobs may migrate.
        // BEAN->GLUE without glue_source must not treat bean trigger_param as script.
        let was_already_glue = self.run_mode.is_glue();
        if let Some(key) = job_param.key {
            self.key = key;
        }
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
        if let Some(glue_source) = job_param.glue_source {
            self.glue_source = glue_source;
        } else if was_already_glue && self.glue_source.is_empty() && self.run_mode.is_glue() {
            // Already-GLUE legacy only: migrate script from trigger_param.
            // Mode switch to GLUE without glue_source is rejected by check_valid.
            if !self.trigger_param.is_empty() {
                self.glue_source = self.trigger_param.clone();
                self.trigger_param = EMPTY_ARC_STR.clone();
            }
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
        if let Some(retry_interval) = job_param.retry_interval {
            self.retry_interval = retry_interval;
        }
        if let Some(update_time) = job_param.update_time {
            self.last_modified_millis = update_time;
            if self.create_time == 0 {
                self.create_time = update_time;
            }
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
        } else if self.run_mode.is_glue() && self.glue_source.is_empty() {
            Err(anyhow::anyhow!("glue_source is empty!"))
        } else if self.schedule_type == ScheduleType::Cron
            && !CronUtil::check_cron_valid(&self.cron_value)
        {
            Err(anyhow::anyhow!("cron_value is invalid!"))
        } else if self.schedule_type == ScheduleType::Interval && self.interval_second == 0 {
            Err(anyhow::anyhow!("interval_second eq 0,it is invalid!"))
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

    pub fn build_job_key(&self) -> JobKey {
        JobKey::new_by_arc(
            self.namespace.clone(),
            self.app_name.clone(),
            self.key.clone(),
        )
    }

    pub fn is_valid(&self) -> bool {
        if self.id == 0 || self.namespace.is_empty() || self.app_name.is_empty() {
            false
        } else if self.run_mode == JobRunMode::Bean && self.handle_name.is_empty() {
            false
        } else if self.run_mode.is_glue()
            && self.glue_source.is_empty()
            && self.trigger_param.is_empty()
        {
            // Unmigrated legacy GLUE keeps the script in trigger_param; that remains
            // valid (JobRunParam falls back at dispatch). Align with dispatch_trigger_param.
            false
        } else if self.schedule_type == ScheduleType::Cron
            && !CronUtil::check_cron_valid(&self.cron_value)
        {
            false
        } else {
            true
        }
    }

    /// Params to use as executorParams when no per-trigger override is set.
    /// Legacy GLUE jobs keep the script in trigger_param while glue_source is empty;
    /// that value must not be sent as executor args (JobRunParam falls back for glueSource).
    pub fn dispatch_trigger_param(&self) -> Arc<String> {
        if self.run_mode.is_glue() && self.glue_source.is_empty() {
            EMPTY_ARC_STR.clone()
        } else {
            self.trigger_param.clone()
        }
    }

    pub fn to_do(&self) -> JobDo<'_> {
        JobDo {
            id: self.id,
            enable: self.enable,
            app_name: Cow::Borrowed(&self.app_name),
            namespace: Cow::Borrowed(&self.namespace),
            job_key: Cow::Borrowed(&self.key),
            description: Cow::Borrowed(&self.description),
            schedule_type: Cow::Borrowed(self.schedule_type.to_str()),
            cron_value: Cow::Borrowed(&self.cron_value),
            delay_second: self.delay_second,
            interval_second: self.interval_second,
            run_mode: Cow::Borrowed(self.run_mode.to_str()),
            handle_name: Cow::Borrowed(&self.handle_name),
            trigger_param: Cow::Borrowed(&self.trigger_param),
            glue_source: Cow::Borrowed(&self.glue_source),
            router_strategy: Cow::Borrowed(self.router_strategy.to_str()),
            past_due_strategy: Cow::Borrowed(self.past_due_strategy.to_str()),
            blocking_strategy: Cow::Borrowed(self.blocking_strategy.to_str()),
            timeout_second: self.timeout_second,
            try_times: self.try_times,
            version_id: self.version_id,
            last_modified_millis: self.last_modified_millis,
            create_time: self.create_time,
            retry_interval: self.retry_interval,
        }
    }
}

/// Create/import normalize: when GLUE and glue_source omitted, move trigger_param into glue_source.
fn migrate_glue_fields_on_create(
    run_mode: &JobRunMode,
    glue_source: Option<Arc<String>>,
    trigger_param: Arc<String>,
) -> (Arc<String>, Arc<String>) {
    match glue_source {
        Some(value) => (value, trigger_param),
        None if run_mode.is_glue() && !trigger_param.is_empty() => {
            (trigger_param, EMPTY_ARC_STR.clone())
        }
        None => (EMPTY_ARC_STR.clone(), trigger_param),
    }
}

impl<'a> From<JobDo<'a>> for JobInfo {
    fn from(job_do: JobDo<'a>) -> Self {
        // Load storage as-is. Legacy jobs without glue_source keep the script in
        // trigger_param; JobRunParam falls back at dispatch for unsaved legacy jobs.
        JobInfo {
            id: job_do.id,
            enable: job_do.enable,
            app_name: Arc::new(job_do.app_name.to_string()),
            namespace: Arc::new(job_do.namespace.to_string()),
            key: Arc::new(job_do.job_key.to_string()),
            description: Arc::new(job_do.description.to_string()),
            schedule_type: ScheduleType::from_str(&job_do.schedule_type),
            cron_value: Arc::new(job_do.cron_value.to_string()),
            delay_second: job_do.delay_second,
            interval_second: job_do.interval_second,
            run_mode: JobRunMode::from_str(&job_do.run_mode).unwrap_or(JobRunMode::Bean),
            handle_name: Arc::new(job_do.handle_name.to_string()),
            trigger_param: Arc::new(job_do.trigger_param.to_string()),
            glue_source: Arc::new(job_do.glue_source.to_string()),
            router_strategy: RouterStrategy::from_str(&job_do.router_strategy)
                .unwrap_or(RouterStrategy::RoundRobin),
            past_due_strategy: PastDueStrategy::from_str(&job_do.past_due_strategy),
            blocking_strategy: ExecutorBlockStrategy::from_str(&job_do.blocking_strategy),
            timeout_second: job_do.timeout_second,
            try_times: job_do.try_times,
            version_id: job_do.version_id,
            last_modified_millis: job_do.last_modified_millis,
            create_time: job_do.create_time,
            retry_interval: job_do.retry_interval,
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

    pub fn update_task_log(
        &mut self,
        new_task_log: Arc<JobTaskInfo>,
        limit_count: usize,
    ) -> Option<bool> {
        if let Some(task_log) = self.task_log_map.get_mut(&new_task_log.task_id) {
            if task_log.status.is_finish() && new_task_log.status.is_running() {
                //先收到sdk的响应再收到内部运行中状态
                return Some(task_log.status == TaskStatusType::Success);
            }
            *task_log = new_task_log;
        } else {
            self.task_log_map.insert(new_task_log.task_id, new_task_log);
            if self.task_log_map.len() > limit_count {
                self.task_log_map.pop_first();
            }
        };
        None
    }
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct JobKey {
    pub namespace: Arc<String>,
    pub app_name: Arc<String>,
    pub job_key: Arc<String>,
}

impl JobKey {
    pub fn new(namespace: &str, app_name: &str, job_key: &str) -> JobKey {
        JobKey {
            namespace: Arc::new(namespace.to_owned()),
            app_name: Arc::new(app_name.to_owned()),
            job_key: Arc::new(job_key.to_owned()),
        }
    }

    pub fn new_by_arc(
        namespace: Arc<String>,
        app_name: Arc<String>,
        job_key: Arc<String>,
    ) -> JobKey {
        JobKey {
            namespace,
            app_name,
            job_key,
        }
    }

    ///
    /// 是否合法的key
    /// 暂时只用于接口层面判断,以支持对部分场景不校验
    ///
    pub fn is_valid(&self) -> anyhow::Result<()> {
        if self.job_key.is_empty() {
            return Err(anyhow::anyhow!(
                "the job_key is invalid : {}",
                self.job_key.as_str()
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobParam {
    pub id: Option<u64>,
    pub enable: Option<bool>,
    pub app_name: Option<Arc<String>>,
    pub namespace: Option<Arc<String>>,
    pub key: Option<Arc<String>>,
    pub description: Option<Arc<String>>,
    pub schedule_type: Option<ScheduleType>,
    pub cron_value: Option<Arc<String>>,
    pub delay_second: Option<u32>,
    pub interval_second: Option<u32>,
    pub run_mode: Option<JobRunMode>,
    pub handle_name: Option<Arc<String>>,
    pub trigger_param: Option<Arc<String>>,
    pub glue_source: Option<Arc<String>>,
    pub router_strategy: Option<RouterStrategy>,
    pub past_due_strategy: Option<PastDueStrategy>,
    pub blocking_strategy: Option<ExecutorBlockStrategy>,
    pub timeout_second: Option<u32>,
    pub try_times: Option<u32>,
    pub update_time: Option<u64>,
    pub retry_interval: Option<u32>,
}

impl JobParam {
    /// Import/historical: GLUE jobs that only have trigger_param as the script.
    /// Moves trigger_param into glue_source and clears trigger_param.
    pub fn migrate_legacy_glue_source(&mut self) {
        let is_glue = self
            .run_mode
            .as_ref()
            .map(JobRunMode::is_glue)
            .unwrap_or(false);
        if !is_glue {
            return;
        }
        let glue_empty = match self.glue_source.as_ref() {
            None => true,
            Some(value) => value.is_empty(),
        };
        if !glue_empty {
            return;
        }
        if let Some(trigger_param) = self.trigger_param.as_ref() {
            if !trigger_param.is_empty() {
                self.glue_source = Some(trigger_param.clone());
                self.trigger_param = Some(EMPTY_ARC_STR.clone());
            }
        }
    }

    pub fn check_valid(&self) -> anyhow::Result<()> {
        if StringUtils::is_option_empty_arc(&self.namespace)
            || StringUtils::is_option_empty_arc(&self.app_name)
        {
            return Err(anyhow::anyhow!("namespace or app_name is empty!"));
        }
        if let Some(run_mode) = self.run_mode.as_ref() {
            if run_mode == &JobRunMode::Bean && StringUtils::is_option_empty_arc(&self.handle_name)
            {
                return Err(anyhow::anyhow!("bean handle_name is invalid!"));
            }
            if run_mode.is_glue() {
                // Explicit empty glue_source is invalid. Omitted glue_source may still
                // migrate from trigger_param on create/update; JobInfo::check_valid is final.
                if let Some(ref glue_source) = self.glue_source {
                    if glue_source.is_empty() {
                        return Err(anyhow::anyhow!("glue_source is empty!"));
                    }
                }
            }
        }
        if let Some(schedule_type) = self.schedule_type.as_ref() {
            if schedule_type == &ScheduleType::Interval
                && self.interval_second.clone().unwrap_or_default() == 0
            {
                return Err(anyhow::anyhow!("interval_second is 0,it is invalid!"));
            }
            if schedule_type == &ScheduleType::Cron {
                if let Some(v) = self.cron_value.as_ref() {
                    if !CronUtil::check_cron_valid(v) {
                        return Err(anyhow::anyhow!("cron_value is invalid!"));
                    }
                } else {
                    return Err(anyhow::anyhow!("cron_value is invalid!"));
                }
            }
        }
        Ok(())
    }
}

impl From<JobParam> for JobInfo {
    fn from(job_param: JobParam) -> Self {
        let run_mode = job_param.run_mode.clone().unwrap_or(JobRunMode::Bean);
        let raw_trigger = job_param
            .trigger_param
            .clone()
            .unwrap_or(EMPTY_ARC_STR.clone());
        let (glue_source, trigger_param) =
            migrate_glue_fields_on_create(&run_mode, job_param.glue_source.clone(), raw_trigger);
        JobInfo {
            id: job_param.id.unwrap_or_default(),
            enable: job_param.enable.unwrap_or(true),
            app_name: job_param.app_name.unwrap_or_default(),
            namespace: job_param.namespace.unwrap_or_default(),
            key: job_param.key.unwrap_or_default(),
            handle_name: job_param.handle_name.unwrap_or(EMPTY_ARC_STR.clone()),
            description: job_param.description.unwrap_or(EMPTY_ARC_STR.clone()),
            schedule_type: job_param.schedule_type.unwrap_or(ScheduleType::None),
            cron_value: job_param.cron_value.unwrap_or(EMPTY_ARC_STR.clone()),
            delay_second: job_param.delay_second.unwrap_or_default(),
            interval_second: job_param.interval_second.unwrap_or_default(),
            run_mode,
            trigger_param,
            glue_source,
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
            retry_interval: job_param.interval_second.unwrap_or(0),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobTaskLogQueryParam {
    pub job_id: u64,
    pub offset: usize,
    pub limit: usize,
    pub namespace: Option<String>,
    pub app_name: Option<String>,
    pub start_trigger_time: Option<u32>,
    pub end_trigger_time: Option<u32>,
    pub status: Option<TaskStatusType>,
}

impl JobTaskLogQueryParam {
    pub fn matches_latest_history(&self, task_log: &JobTaskInfo) -> bool {
        if let Some(ref namespace) = self.namespace {
            if task_log.namespace.as_ref() != namespace {
                return false;
            }
        }
        if let Some(ref app_name) = self.app_name {
            if task_log.app_name.as_ref() != app_name {
                return false;
            }
        }
        if let Some(start_trigger_time) = self.start_trigger_time {
            if task_log.trigger_time < start_trigger_time {
                return false;
            }
        }
        if let Some(end_trigger_time) = self.end_trigger_time {
            if task_log.trigger_time > end_trigger_time {
                return false;
            }
        }
        if let Some(ref status) = self.status {
            if &task_log.status != status {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfoDto {
    pub id: u64,
    pub enable: bool,
    pub app_name: Arc<String>,
    pub key: Arc<String>,
    pub namespace: Arc<String>,
    pub description: Arc<String>,
    pub schedule_type: String,
    pub cron_value: Arc<String>,
    pub delay_second: u32,
    pub interval_second: u32,
    pub run_mode: String,
    pub handle_name: Arc<String>,
    pub trigger_param: Arc<String>,
    #[serde(default)]
    pub glue_source: Arc<String>,
    pub router_strategy: String,
    pub past_due_strategy: String,
    pub blocking_strategy: String,
    pub timeout_second: u32,
    pub try_times: u32,
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub register_time: u64,
    pub retry_interval: u32,
}

impl JobInfoDto {
    pub fn new_from(job_info: &JobInfo) -> Self {
        JobInfoDto {
            id: job_info.id,
            enable: job_info.enable,
            app_name: job_info.app_name.clone(),
            key: job_info.key.clone(),
            namespace: job_info.namespace.clone(),
            description: job_info.description.clone(),
            schedule_type: job_info.schedule_type.to_str().to_owned(),
            cron_value: job_info.cron_value.clone(),
            delay_second: job_info.delay_second,
            interval_second: job_info.interval_second,
            run_mode: job_info.run_mode.to_str().to_owned(),
            handle_name: job_info.handle_name.clone(),
            trigger_param: job_info.trigger_param.clone(),
            glue_source: job_info.glue_source.clone(),
            router_strategy: job_info.router_strategy.to_str().to_owned(),
            past_due_strategy: job_info.past_due_strategy.to_str().to_owned(),
            blocking_strategy: job_info.blocking_strategy.to_str().to_owned(),
            timeout_second: job_info.timeout_second,
            try_times: job_info.try_times,
            version_id: job_info.version_id,
            last_modified_millis: job_info.last_modified_millis,
            register_time: job_info.create_time,
            retry_interval: job_info.retry_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{JobInfo, JobParam};
    use crate::common::constant::EMPTY_ARC_STR;
    use crate::common::pb::data_object::JobDo;
    use crate::job::model::enum_type::JobRunMode;
    use quick_protobuf::{BytesReader, Writer};
    use std::borrow::Cow;
    use std::sync::Arc;

    fn stored_job(run_mode: &str, trigger_param: &str, glue_source: &str) -> JobDo<'static> {
        JobDo {
            run_mode: Cow::Owned(run_mode.to_string()),
            trigger_param: Cow::Owned(trigger_param.to_string()),
            glue_source: Cow::Owned(glue_source.to_string()),
            ..JobDo::default()
        }
    }

    #[test]
    fn old_glue_job_keeps_script_in_trigger_param_on_load() {
        // No silent rewrite on protobuf load; dispatch falls back at JobRunParam.
        let info = JobInfo::from(stored_job("GLUE_GROOVY", "println 1", ""));
        assert_eq!(info.glue_source.as_str(), "");
        assert_eq!(info.trigger_param.as_str(), "println 1");
    }

    #[test]
    fn stored_glue_source_and_param_stay_separate() {
        let info = JobInfo::from(stored_job("GLUE_GROOVY", "arg", "println 1"));
        assert_eq!(info.glue_source.as_str(), "println 1");
        assert_eq!(info.trigger_param.as_str(), "arg");
    }

    #[test]
    fn bean_job_does_not_copy_param_into_script() {
        let info = JobInfo::from(stored_job("BEAN", "arg", ""));
        assert_eq!(info.glue_source.as_str(), "");
        assert_eq!(info.trigger_param.as_str(), "arg");
    }

    #[test]
    fn legacy_glue_dispatch_trigger_param_is_empty() {
        let info = JobInfo::from(stored_job("GLUE_GROOVY", "println 1", ""));
        assert_eq!(info.dispatch_trigger_param().as_str(), "");
    }

    #[test]
    fn migrated_glue_dispatch_keeps_real_trigger_param() {
        let info = JobInfo::from(stored_job("GLUE_GROOVY", "arg", "println 1"));
        assert_eq!(info.dispatch_trigger_param().as_str(), "arg");
    }

    #[test]
    fn bean_dispatch_uses_trigger_param() {
        let info = JobInfo::from(stored_job("BEAN", "arg", ""));
        assert_eq!(info.dispatch_trigger_param().as_str(), "arg");
    }

    #[test]
    fn roundtrip_keeps_param_and_script_separate() {
        let info = JobInfo {
            id: 7,
            run_mode: JobRunMode::GlueShell,
            trigger_param: Arc::new("arg".to_string()),
            glue_source: Arc::new("echo hi".to_string()),
            ..JobInfo::default()
        };
        let record = info.to_do();
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        writer.write_message(&record).unwrap();
        let mut reader = BytesReader::from_bytes(&buf);
        let loaded: JobDo = reader.read_message(&buf).unwrap();
        let restored = JobInfo::from(loaded);
        assert_eq!(restored.trigger_param.as_str(), "arg");
        assert_eq!(restored.glue_source.as_str(), "echo hi");
    }

    #[test]
    fn omitted_glue_source_on_glue_create_migrates_and_clears_trigger_param() {
        let info = JobInfo::from(JobParam {
            run_mode: Some(JobRunMode::GluePython),
            trigger_param: Some(Arc::new("print(1)".to_string())),
            ..JobParam::default()
        });
        assert_eq!(info.glue_source.as_str(), "print(1)");
        assert_eq!(info.trigger_param.as_str(), "");
    }

    #[test]
    fn explicit_empty_glue_source_fails_validation() {
        let info = JobInfo::from(JobParam {
            id: Some(1),
            namespace: Some(Arc::new("ns".to_string())),
            app_name: Some(Arc::new("app".to_string())),
            run_mode: Some(JobRunMode::GluePython),
            trigger_param: Some(Arc::new("print(1)".to_string())),
            glue_source: Some(Arc::new(String::new())),
            ..JobParam::default()
        });
        assert_eq!(info.glue_source.as_str(), "");
        assert_eq!(info.trigger_param.as_str(), "print(1)");
        assert!(info.check_valid().is_err());
    }

    #[test]
    fn import_legacy_glue_job_migrates_empty_glue_source() {
        let mut param = JobParam {
            run_mode: Some(JobRunMode::GlueShell),
            trigger_param: Some(Arc::new("echo hi".to_string())),
            glue_source: Some(Arc::new(String::new())),
            ..JobParam::default()
        };
        param.migrate_legacy_glue_source();
        let info = JobInfo::from(param);
        assert_eq!(info.glue_source.as_str(), "echo hi");
        assert_eq!(info.trigger_param.as_str(), "");
    }

    #[test]
    fn update_omitted_glue_source_migrates_old_script_only_job() {
        let mut info = JobInfo {
            id: 1,
            namespace: Arc::new("ns".to_string()),
            app_name: Arc::new("app".to_string()),
            run_mode: JobRunMode::GlueShell,
            trigger_param: Arc::new("echo hi".to_string()),
            glue_source: EMPTY_ARC_STR.clone(),
            ..JobInfo::default()
        };
        info.update_param(JobParam {
            description: Some(Arc::new("migrated".to_string())),
            ..JobParam::default()
        });
        assert_eq!(info.glue_source.as_str(), "echo hi");
        assert_eq!(info.trigger_param.as_str(), "");
        assert_eq!(info.description.as_str(), "migrated");
        assert!(info.check_valid().is_ok());
    }

    #[test]
    fn legacy_glue_with_script_in_trigger_param_is_valid() {
        let mut info = JobInfo::from(stored_job("GLUE_GROOVY", "println 1", ""));
        info.id = 1;
        info.namespace = Arc::new("ns".to_string());
        info.app_name = Arc::new("app".to_string());
        assert!(info.is_valid());
        assert_eq!(info.dispatch_trigger_param().as_str(), "");
    }

    #[test]
    fn glue_with_both_script_fields_empty_is_invalid() {
        let mut info = JobInfo::from(stored_job("GLUE_GROOVY", "", ""));
        info.id = 1;
        info.namespace = Arc::new("ns".to_string());
        info.app_name = Arc::new("app".to_string());
        assert!(!info.is_valid());
    }

    #[test]
    fn update_bean_to_glue_without_glue_source_fails() {
        let mut info = JobInfo {
            id: 1,
            namespace: Arc::new("ns".to_string()),
            app_name: Arc::new("app".to_string()),
            run_mode: JobRunMode::Bean,
            handle_name: Arc::new("demoHandler".to_string()),
            trigger_param: Arc::new("bean-args".to_string()),
            glue_source: EMPTY_ARC_STR.clone(),
            ..JobInfo::default()
        };
        info.update_param(JobParam {
            run_mode: Some(JobRunMode::GlueShell),
            // glue_source omitted — must not migrate bean trigger_param into script
            ..JobParam::default()
        });
        assert_eq!(info.run_mode, JobRunMode::GlueShell);
        assert_eq!(info.glue_source.as_str(), "");
        assert_eq!(info.trigger_param.as_str(), "bean-args");
        let err = info.check_valid().unwrap_err();
        assert!(
            err.to_string().contains("glue_source"),
            "expected glue_source validation error, got: {err}"
        );
    }
}
