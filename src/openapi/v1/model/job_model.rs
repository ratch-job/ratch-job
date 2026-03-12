use crate::job::model::job::{JobKey, JobTaskLogQueryParam};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobTaskListRequest {
    pub job_id: Option<u64>,
    pub page_no: Option<usize>,
    pub page_size: Option<usize>,
}

impl JobTaskListRequest {
    pub fn to_param(self) -> JobTaskLogQueryParam {
        let limit = self.page_size.unwrap_or(10);
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

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobTaskHistoryRequest {
    pub job_id: Option<u64>,
    pub page_no: Option<usize>,
    pub page_size: Option<usize>,
}

impl JobTaskHistoryRequest {
    pub fn to_param(self) -> JobTaskLogQueryParam {
        let limit = self.page_size.unwrap_or(10);
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

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobKeyQueryRequest {
    pub namespace: Option<String>,
    pub app_name: Option<String>,
    pub job_key: Option<String>,
}

impl JobKeyQueryRequest {
    pub fn to_job_key(self) -> Option<JobKey> {
        let namespace = self.namespace?;
        let app_name = self.app_name?;
        let job_key = self.job_key?;

        if namespace.is_empty() || app_name.is_empty() || job_key.is_empty() {
            return None;
        }

        Some(JobKey::new(&namespace, &app_name, &job_key))
    }
}
