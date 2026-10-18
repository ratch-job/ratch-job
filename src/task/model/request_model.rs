use crate::job::model::job::JobInfo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobRunParam {
    pub job_id: u64,
    pub log_id: u64,
    pub executor_handler: Option<Arc<String>>,
    pub executor_params: Option<Arc<String>>,
    pub executor_block_strategy: Option<String>,
    pub executor_timeout: Option<i32>,
    pub log_date_time: Option<u64>,
    pub glue_type: Option<String>,
    pub glue_source: Option<Arc<String>>,
    #[serde(rename(serialize = "glueUpdatetime", deserialize = "glueUpdatetime"))]
    pub glue_update_time: Option<u64>,
    pub broadcast_index: Option<u64>,
    pub broadcast_total: Option<u64>,
}

impl JobRunParam {
    pub fn from_job_info(
        log_id: u64,
        job_info: &Arc<JobInfo>,
        executor_params: Arc<String>,
    ) -> Self {
        Self {
            job_id: job_info.id,
            log_id,
            executor_handler: Some(job_info.handle_name.clone()),
            executor_params: Some(executor_params),
            executor_block_strategy: Some(job_info.blocking_strategy.to_str().to_string()),
            executor_timeout: Some(job_info.timeout_second as i32),
            log_date_time: Some(job_info.last_modified_millis),
            glue_type: Some(job_info.run_mode.to_str().to_string()),
            glue_source: Some(
                if job_info.glue_source.is_empty() && job_info.run_mode.is_glue() {
                    // Unsaved legacy GLUE jobs: script still lives in trigger_param.
                    job_info.trigger_param.clone()
                } else {
                    job_info.glue_source.clone()
                },
            ),
            glue_update_time: Some(job_info.last_modified_millis),
            broadcast_index: Some(0),
            broadcast_total: Some(0),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobLogParam {
    pub log_id: u64,
    #[serde(rename(serialize = "logDateTim", deserialize = "logDateTim"))]
    pub log_date_time: Option<u64>,
    pub from_line_num: i64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobLogInfo {
    pub log_content: String,
    pub from_line_num: i64,
    pub to_line_num: i64,
    pub is_end: bool,
}

#[cfg(test)]
mod tests {
    use super::{JobLogInfo, JobLogParam};

    #[test]
    fn should_serialize_xxl_job_242_log_param() {
        let param = JobLogParam {
            log_id: 12,
            log_date_time: Some(1_789_891_683_000),
            from_line_num: 3,
        };

        let value = serde_json::to_value(param).expect("log param should serialize");

        assert_eq!(value["logDateTim"], 1_789_891_683_000_u64);
        assert_eq!(value["logId"], 12);
        assert_eq!(value["fromLineNum"], 3);
        assert!(value.get("logDateTime").is_none());
    }

    #[test]
    fn should_deserialize_xxl_job_242_log_result() {
        let value = br#"{
            "fromLineNum": 1,
            "toLineNum": 8,
            "logContent": "hello\n",
            "isEnd": false
        }"#;

        let info: JobLogInfo =
            serde_json::from_slice(value).expect("log result should deserialize");

        assert_eq!(info.from_line_num, 1);
        assert_eq!(info.to_line_num, 8);
        assert_eq!(info.log_content, "hello\n");
        assert!(!info.is_end);
    }

    #[test]
    fn run_param_keeps_executor_param_separate_from_glue_script() {
        use crate::job::model::enum_type::JobRunMode;
        use crate::job::model::job::JobInfo;
        use std::sync::Arc;

        let job = Arc::new(JobInfo {
            id: 3,
            run_mode: JobRunMode::GlueShell,
            handle_name: Arc::new("demo".to_string()),
            trigger_param: Arc::new("saved".to_string()),
            glue_source: Arc::new("echo hi".to_string()),
            ..JobInfo::default()
        });
        let param = super::JobRunParam::from_job_info(9, &job, Arc::new("once".to_string()));
        let value = serde_json::to_value(param).expect("run param should serialize");

        assert_eq!(value["executorParams"], "once");
        assert_eq!(value["glueSource"], "echo hi");
        assert_eq!(value["glueType"], "GLUE_SHELL");
    }

    #[test]
    fn run_param_falls_back_to_trigger_param_when_glue_source_empty() {
        use crate::job::model::enum_type::JobRunMode;
        use crate::job::model::job::JobInfo;
        use std::sync::Arc;

        let job = Arc::new(JobInfo {
            id: 4,
            run_mode: JobRunMode::GlueGroovy,
            handle_name: Arc::new("demo".to_string()),
            trigger_param: Arc::new("println 1".to_string()),
            glue_source: Arc::new(String::new()),
            ..JobInfo::default()
        });
        // Caller must not dual-use the legacy script as executorParams.
        let param = super::JobRunParam::from_job_info(10, &job, Arc::new(String::new()));
        let value = serde_json::to_value(param).expect("run param should serialize");

        assert_eq!(value["executorParams"], "");
        assert_eq!(value["glueSource"], "println 1");
        assert_eq!(value["glueType"], "GLUE_GROOVY");
    }

    #[test]
    fn bean_run_param_does_not_fallback_glue_source() {
        use crate::job::model::enum_type::JobRunMode;
        use crate::job::model::job::JobInfo;
        use std::sync::Arc;

        let job = Arc::new(JobInfo {
            id: 5,
            run_mode: JobRunMode::Bean,
            handle_name: Arc::new("demo".to_string()),
            trigger_param: Arc::new("arg".to_string()),
            glue_source: Arc::new(String::new()),
            ..JobInfo::default()
        });
        let param = super::JobRunParam::from_job_info(11, &job, Arc::new("arg".to_string()));
        let value = serde_json::to_value(param).expect("run param should serialize");

        assert_eq!(value["executorParams"], "arg");
        assert_eq!(value["glueSource"], "");
        assert_eq!(value["glueType"], "BEAN");
    }
}
