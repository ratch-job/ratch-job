use std::sync::Arc;
pub const EMPTY_STR: &str = "";

pub const HTTP_METHOD_GET: &str = "GET";
//pub const HTTP_METHOD_PUT:&str= "PUT";
//pub const HTTP_METHOD_POST:&str= "POST";
//pub const HTTP_METHOD_DELETE:&str= "DELETE";
pub const HTTP_METHOD_ALL: &str = EMPTY_STR;

pub const CONSOLE_TOKEN_COOKIE_KEY: &str = "ratch_job_token";

lazy_static::lazy_static! {
    pub static ref EMPTY_ARC_STR: Arc<String> =  Arc::new("".to_string());
    pub static ref TRIGGER_FROM_SYSTEM: Arc<String> =  Arc::new("_".to_string());
    pub static ref DEFAULT_NAMESPACE: Arc<String> =  Arc::new("default".to_string());
    pub static ref DEFAULT_XXL_NAMESPACE: Arc<String> =  Arc::new("xxl".to_string());

    pub static ref SEQUENCE_TABLE_NAME: Arc<String> =  Arc::new("T_SEQUENCE".to_string());
    pub static ref APP_INFO_TABLE_NAME: Arc<String> =  Arc::new("T_APP_INFO".to_string());
    pub static ref JOB_TABLE_NAME: Arc<String> =  Arc::new("T_JOB".to_string());
    pub static ref JOB_TASK_TABLE_NAME: Arc<String> =  Arc::new("T_JOB_TASK".to_string());
    pub static ref JOB_TASK_RUNNING_TABLE_NAME: Arc<String> =  Arc::new("T_JOB_TASK_RUNNING".to_string());
    pub static ref JOB_TASK_HISTORY_TABLE_NAME: Arc<String> =  Arc::new("T_JOB_TASK_HISTORY".to_string());
    pub static ref CACHE_TABLE_NAME: Arc<String> =  Arc::new("T_CACHE".to_string());
    pub static ref USER_TABLE_NAME: Arc<String> =  Arc::new("T_USER".to_string());

    pub static ref SEQ_JOB_ID: Arc<String> =  Arc::new("job_id".to_string());
    pub static ref SEQ_TASK_ID: Arc<String> =  Arc::new("task_id".to_string());


    // error info
    pub static ref ERR_MSG_NOT_FOUND_APP_INSTANCE_ADDR: Arc<String> =  Arc::new("Not found the application instance address".to_string());
    pub static ref ERR_MSG_JOB_DISABLE: Arc<String> =  Arc::new("Job is disabled or not found".to_string());
    pub static ref ERR_MSG_TASK_TIMEOUT: Arc<String> =  Arc::new("Task timed out".to_string());
}
