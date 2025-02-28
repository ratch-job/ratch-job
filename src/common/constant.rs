use std::sync::Arc;
pub const EMPTY_STR: &str = "";

lazy_static::lazy_static! {
    pub static ref EMPTY_ARC_STR: Arc<String> =  Arc::new("".to_string());
    pub static ref DEFAULT_NAMESPACE: Arc<String> =  Arc::new("default".to_string());
    pub static ref DEFAULT_XXL_NAMESPACE: Arc<String> =  Arc::new("xxl".to_string());

    pub static ref SEQUENCE_TABLE_NAME: Arc<String> =  Arc::new("T_SEQUENCE".to_string());

    pub static ref SEQ_JOB_ID: Arc<String> =  Arc::new("job_id".to_string());
    pub static ref SEQ_TASK_ID: Arc<String> =  Arc::new("task_id".to_string());
}
