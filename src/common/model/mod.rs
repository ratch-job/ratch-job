pub mod privilege;

use std::{collections::HashMap, sync::Arc};

use crate::common::model::privilege::PrivilegeGroup;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiResult<T>
where
    T: Sized,
{
    pub data: Option<T>,
    pub success: bool,
    pub code: Option<String>,
    pub message: Option<String>,
}

impl<T> ApiResult<T>
where
    T: Sized,
{
    pub fn success(data: Option<T>) -> Self {
        Self {
            data,
            success: true,
            code: None,
            message: None,
        }
    }

    pub fn error(code: String, message: Option<String>) -> Self {
        Self {
            data: None,
            success: false,
            code: Some(code),
            message,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResult<T> {
    pub total_count: usize,
    pub list: Vec<T>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct UserSession {
    pub username: Arc<String>,
    pub nickname: String,
    pub roles: Vec<Arc<String>>,
    pub extend_infos: HashMap<String, String>,
    pub namespace_privilege: PrivilegeGroup<Arc<String>>,
    pub app_privilege: PrivilegeGroup<Arc<String>>,
    /// 时间戳，单位秒
    pub refresh_time: u32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct TokenSession {
    pub username: Arc<String>,
    pub roles: Vec<Arc<String>>,
    pub extend_infos: HashMap<String, String>,
}
