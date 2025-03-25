use crate::openapi::xxljob::model::SUCCESS_CODE;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistryParam {
    pub registry_group: Arc<String>,
    pub registry_key: Arc<String>,
    pub registry_value: Arc<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallbackParam {
    pub log_id: u64,
    #[serde(rename(serialize = "logDateTim", deserialize = "logDateTim"))]
    pub log_date_time: i64,
    pub handle_code: i32,
    pub handle_msg: Option<Arc<String>>,
}

impl From<CallbackParam> for crate::task::model::task::TaskCallBackParam {
    fn from(value: CallbackParam) -> Self {
        Self {
            task_id: value.log_id,
            task_date_time: value.log_date_time,
            success: value.handle_code == SUCCESS_CODE,
            handle_msg: value.handle_msg,
        }
    }
}
