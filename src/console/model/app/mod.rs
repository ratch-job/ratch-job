use crate::app::app_index::AppQueryParam;
use crate::app::model::{AppParam, RegisterType};
use crate::common::namespace_util::get_namespace_by_option;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoParam {
    pub namespace: Option<Arc<String>>,
    pub app_name: Option<Arc<String>>,
    pub label: Option<Arc<String>>,
    pub register_type: Option<String>,
    pub instance_addrs: Option<Vec<Arc<String>>>,
}

impl AppInfoParam {
    pub fn to_param(self) -> AppParam {
        AppParam {
            app_name: self.app_name.unwrap_or_default(),
            namespace: get_namespace_by_option(&self.namespace),
            label: self.label,
            register_type: self.register_type.map(|s| RegisterType::from_str(&s)),
            instance_addrs: self.instance_addrs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppQueryListRequest {
    pub namespace: Option<Arc<String>>,
    pub app_name: Option<Arc<String>>,
    pub like_app_name: Option<String>,
    pub page_no: Option<usize>,
    pub page_size: Option<usize>,
}

impl AppQueryListRequest {
    pub fn to_param(self) -> AppQueryParam {
        let limit = self.page_size.unwrap_or(0xffff_ffff);
        let page_no = if self.page_no.unwrap_or(1) < 1 {
            1
        } else {
            self.page_no.unwrap_or(1)
        };
        let offset = (page_no - 1) * limit;
        AppQueryParam {
            namespace: self.namespace,
            app_name: self.app_name,
            like_name: self.like_app_name,
            offset,
            limit,
        }
    }
}
