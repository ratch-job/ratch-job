use std::borrow::Cow;
use std::collections::HashMap;
//use crate::metrics::model::MetricsType;
use lazy_static::lazy_static;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Label(pub Cow<'static, str>, pub Cow<'static, str>);

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum MetricsKey {
    // app
    ProcessStartTimeSeconds,
    SysTotalMemory,
    AppRssMemory,
    AppVmsMemory,
    AppMemoryUsage,
    AppCpuUsage,
    // job app instance
    JobAppSize,
    JobAppInstanceSize,
    // job
    JobSize,
    JobEnableSize,
    // schedule task
    TaskTriggerSize,
    TaskRedoSize,
    TaskSuccessSize,
    TaskFailSize,
    TaskRunningSize,
    TaskCallApiSize,
    TaskFinishRtHistogram,
    TaskFinishRtSummary,
    TaskFinishTotalCount,
    //http api request
    HttpRequestHandleRtHistogram,
    HttpRequestHandleRtSummary,
    HttpRequestTotalCount,
}

lazy_static! {
    /// 用于有序遍历打印信息
    pub static ref ORDER_ALL_KEYS: Vec<MetricsKey> = vec![
        //app
        MetricsKey::SysTotalMemory,
        MetricsKey::AppRssMemory,
        MetricsKey::AppVmsMemory,
        MetricsKey::AppMemoryUsage,
        MetricsKey::AppCpuUsage,
        // job app instance
        MetricsKey::JobAppSize,
        MetricsKey::JobAppInstanceSize,
        // job
        MetricsKey::JobSize,
        MetricsKey::JobEnableSize,
        // schedule task
        MetricsKey::TaskTriggerSize,
        MetricsKey::TaskRedoSize,
        MetricsKey::TaskSuccessSize,
        MetricsKey::TaskFailSize,
        MetricsKey::TaskCallApiSize,
        MetricsKey::TaskFinishRtHistogram,
        MetricsKey::TaskFinishRtSummary,
        MetricsKey::TaskFinishTotalCount,
        //http request
        MetricsKey::HttpRequestHandleRtHistogram,
        MetricsKey::HttpRequestHandleRtSummary,
        MetricsKey::HttpRequestTotalCount,
    ];

    pub static ref HISTOGRAM_SUMMARY_MAP: HashMap<MetricsKey,MetricsKey> = MetricsKey::build_histogram_summary_map();

    pub static ref STR_TO_METRICS_KEY_MAP: HashMap<&'static str,MetricsKey> = MetricsKey::build_str_to_key_map();
}

impl MetricsKey {
    pub fn get_key(&self) -> &'static str {
        match &self {
            MetricsKey::ProcessStartTimeSeconds => "process_start_time_seconds",
            MetricsKey::SysTotalMemory => "sys_total_memory",
            MetricsKey::AppRssMemory => "app_rss_memory",
            MetricsKey::AppVmsMemory => "app_vms_memory",
            MetricsKey::AppMemoryUsage => "app_memory_usage",
            MetricsKey::AppCpuUsage => "app_cpu_usage",
            MetricsKey::JobAppSize => "job_app_size",
            MetricsKey::JobAppInstanceSize => "job_app_instance_size",
            MetricsKey::JobSize => "job_size",
            MetricsKey::JobEnableSize => "job_enable_size",
            MetricsKey::TaskTriggerSize => "task_trigger_size",
            MetricsKey::TaskRedoSize => "task_redo_size",
            MetricsKey::TaskSuccessSize => "task_success_size",
            MetricsKey::TaskFailSize => "task_fail_size",
            MetricsKey::TaskRunningSize => "task_running_size",
            MetricsKey::TaskCallApiSize => "task_call_api_size",
            MetricsKey::TaskFinishRtHistogram => "task_finish_rt_histogram",
            MetricsKey::TaskFinishRtSummary => "task_finish_rt_summary",
            MetricsKey::TaskFinishTotalCount => "task_finish_total_count",
            MetricsKey::HttpRequestHandleRtHistogram => "http_request_handle_rt_histogram",
            MetricsKey::HttpRequestHandleRtSummary => "http_request_handle_rt_summary",
            MetricsKey::HttpRequestTotalCount => "http_request_total_count",
        }
    }

    pub fn get_labels(&self) -> Option<Vec<&Label>> {
        //todo 后续的指标key可以支持labels
        None
    }

    pub fn get_key_with_label(&self) -> Cow<'static, str> {
        let key = self.get_key();
        if let Some(_labels) = self.get_labels() {
            //todo 把key与label拼接到一起展示
            //key{label_key=label_value,label_key2=label_value2}
            Cow::Owned(key.to_string())
        } else {
            Cow::Borrowed(key)
        }
    }

    pub fn get_describe(&self) -> &'static str {
        match &self {
            MetricsKey::ProcessStartTimeSeconds => "Process start time seconds",
            MetricsKey::SysTotalMemory => "Sys total memory,unit is M",
            MetricsKey::AppRssMemory => "App rss memory,unit is M",
            MetricsKey::AppVmsMemory => "App vms memory,unit is M",
            MetricsKey::AppMemoryUsage => "App memory usage",
            MetricsKey::AppCpuUsage => "App cpu usage",
            MetricsKey::JobAppSize => "Job app size",
            MetricsKey::JobAppInstanceSize => "Job app instance size",
            MetricsKey::JobSize => "Job size",
            MetricsKey::JobEnableSize => "Job enable size",
            MetricsKey::TaskTriggerSize => "Task trigger size",
            MetricsKey::TaskRedoSize => "Task redo size",
            MetricsKey::TaskSuccessSize => "Task success size",
            MetricsKey::TaskFailSize => "Task fail size",
            MetricsKey::TaskRunningSize => "Task running size",
            MetricsKey::TaskCallApiSize => "Task call api size",
            MetricsKey::TaskFinishRtHistogram => "Task finish rt histogram,unit is ms",
            MetricsKey::TaskFinishRtSummary => "Task finish rt summary,unit is ms",
            MetricsKey::TaskFinishTotalCount => "Task finish total count",
            MetricsKey::HttpRequestHandleRtHistogram => {
                "Http request handle rt histogram,unit is ms"
            }
            MetricsKey::HttpRequestHandleRtSummary => "Http request handle rt summary,unit is ms",
            MetricsKey::HttpRequestTotalCount => "Http request total count",
        }
    }

    pub fn of_key(key: &str) -> Option<Self> {
        STR_TO_METRICS_KEY_MAP.get(key).cloned()
    }

    pub fn get_histogram_from_summary(key: &Self) -> Option<Self> {
        HISTOGRAM_SUMMARY_MAP.get(key).cloned()
    }

    pub fn get_summary_from_histogram(key: &Self) -> Option<Self> {
        HISTOGRAM_SUMMARY_MAP.get(key).cloned()
    }

    fn build_histogram_summary_map() -> HashMap<MetricsKey, MetricsKey> {
        let mut map = HashMap::new();
        map.insert(
            MetricsKey::HttpRequestHandleRtHistogram,
            MetricsKey::HttpRequestHandleRtSummary,
        );
        map.insert(
            MetricsKey::HttpRequestHandleRtSummary,
            MetricsKey::HttpRequestHandleRtHistogram,
        );
        map.insert(
            MetricsKey::TaskFinishRtHistogram,
            MetricsKey::TaskFinishRtSummary,
        );
        map.insert(
            MetricsKey::TaskFinishRtSummary,
            MetricsKey::TaskFinishRtHistogram,
        );
        map
    }

    fn build_str_to_key_map() -> HashMap<&'static str, MetricsKey> {
        let mut map = HashMap::new();
        for key in ORDER_ALL_KEYS.iter() {
            map.insert(key.get_key(), key.to_owned());
        }
        map
    }
}
