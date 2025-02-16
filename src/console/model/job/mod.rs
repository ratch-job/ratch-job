use crate::common::namespace_util::get_namespace_by_option;
use crate::job::job_index::JobQueryParam;
use crate::job::model::enum_type::{
    ExecutorBlockStrategy, JobRunMode, PastDueStrategy, RouterStrategy, ScheduleType,
};
use crate::job::model::job::{JobParam, JobTaskLogQueryParam};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobInfoParam {
    pub id: Option<u64>,
    pub enable: Option<bool>,
    pub namespace: Option<Arc<String>>,
    pub app_name: Option<Arc<String>>,
    pub description: Option<Arc<String>>,
    pub schedule_type: Option<String>,
    pub cron_value: Option<Arc<String>>,
    pub delay_second: Option<u32>,
    pub interval_second: Option<u32>,
    pub run_mode: Option<String>,
    pub handle_name: Option<Arc<String>>,
    pub trigger_param: Option<Arc<String>>,
    pub router_strategy: Option<String>,
    pub past_due_strategy: Option<String>,
    pub blocking_strategy: Option<String>,
    pub timeout_second: Option<u32>,
    pub try_times: Option<u32>,
}

impl JobInfoParam {
    pub fn to_param(self) -> JobParam {
        JobParam {
            id: self.id,
            enable: self.enable,
            namespace: Some(get_namespace_by_option(&self.namespace)),
            app_name: self.app_name,
            description: self.description,
            schedule_type: self.schedule_type.map(|s| ScheduleType::from_str(&s)),
            cron_value: self.cron_value,
            delay_second: self.delay_second,
            interval_second: self.interval_second,
            run_mode: self
                .run_mode
                .map(|s| JobRunMode::from_str(&s).unwrap_or(JobRunMode::Bean)),
            handle_name: self.handle_name,
            trigger_param: self.trigger_param,
            router_strategy: self
                .router_strategy
                .map(|s| RouterStrategy::from_str(&s).unwrap_or(RouterStrategy::RoundRobin)),
            past_due_strategy: self
                .past_due_strategy
                .map(|s| PastDueStrategy::from_str(&s)),
            blocking_strategy: self
                .blocking_strategy
                .map(|s| ExecutorBlockStrategy::from_str(&s)),
            timeout_second: self.timeout_second,
            try_times: self.try_times,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobQueryListRequest {
    pub namespace: Option<Arc<String>>,
    pub app_name: Option<Arc<String>>,
    pub like_description: Option<Arc<String>>,
    pub like_handle_name: Option<Arc<String>>,
    pub page_no: Option<usize>,
    pub page_size: Option<usize>,
}

impl JobQueryListRequest {
    pub fn to_param(self) -> JobQueryParam {
        let limit = self.page_size.unwrap_or(0xffff_ffff);
        let page_no = if self.page_no.unwrap_or(1) < 1 {
            1
        } else {
            self.page_no.unwrap_or(1)
        };
        let offset = (page_no - 1) * limit;
        JobQueryParam {
            namespace: self.namespace,
            app_name: self.app_name,
            like_description: self.like_description,
            like_handle_name: self.like_handle_name,
            offset,
            limit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobTaskLogQueryListRequest {
    pub job_id: Option<u64>,
    pub page_no: Option<usize>,
    pub page_size: Option<usize>,
}

impl JobTaskLogQueryListRequest {
    pub fn to_param(self) -> JobTaskLogQueryParam {
        let limit = self.page_size.unwrap_or(0xffff_ffff);
        let page_no = if self.page_no.unwrap_or(1) < 1 {
            1
        } else {
            self.page_no.unwrap_or(1)
        };
        let offset = (page_no - 1) * limit;
        JobTaskLogQueryParam {
            job_id: self.job_id.unwrap_or_default(),
            offset,
            limit,
        }
    }
}
