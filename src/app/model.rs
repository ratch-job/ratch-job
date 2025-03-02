use crate::app::app_index::AppQueryParam;
use crate::common::datetime_utils::now_millis;
use crate::common::pb::data_object::AppInfoDo;
use actix::Message;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegisterType {
    Auto,
    Manual,
}

impl RegisterType {
    pub fn from_str(s: &str) -> RegisterType {
        match s {
            "AUTO" => RegisterType::Auto,
            "MANUAL" => RegisterType::Manual,
            _ => RegisterType::Auto,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            RegisterType::Auto => "AUTO",
            RegisterType::Manual => "MANUAL",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub label: Arc<String>,
    pub register_type: RegisterType,
    pub tmp: bool,
    pub instance_map: HashMap<Arc<String>, AppInstance>,
}

impl AppInfo {
    pub fn new(
        name: Arc<String>,
        namespace: Arc<String>,
        label: Arc<String>,
        register_type: RegisterType,
        tmp: bool,
    ) -> Self {
        AppInfo {
            app_name: name,
            namespace,
            label,
            register_type,
            instance_map: HashMap::new(),
            tmp,
        }
    }

    pub fn build_key(&self) -> AppKey {
        AppKey::new(self.app_name.clone(), self.namespace.clone())
    }

    pub fn to_do(&self) -> AppInfoDo {
        AppInfoDo {
            app_name: Cow::Borrowed(&self.app_name),
            namespace: Cow::Borrowed(&self.namespace),
            label: Cow::Borrowed(&self.label),
            register_type: Cow::Borrowed(self.register_type.to_str()),
            tmp: self.tmp,
        }
    }
}

impl<'a> From<AppInfoDo<'a>> for AppInfo {
    fn from(record: AppInfoDo<'a>) -> Self {
        AppInfo {
            app_name: Arc::new(record.app_name.to_string()),
            namespace: Arc::new(record.namespace.to_string()),
            label: Arc::new(record.label.to_string()),
            register_type: RegisterType::from_str(&record.register_type),
            instance_map: HashMap::new(),
            tmp: record.tmp,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppInstance {
    pub addr: Arc<String>,
    pub healthy: bool,
    pub enable: bool,
    pub last_modified_millis: u64,
    pub register_time: u64,
}

impl AppInstance {
    pub fn new(addr: Arc<String>) -> Self {
        let now = now_millis();
        AppInstance {
            addr,
            healthy: true,
            enable: true,
            last_modified_millis: now,
            register_time: now,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInstanceDto {
    pub app_key: AppKey,
    pub instance_addr: Arc<String>,
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppKey {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
}

impl AppKey {
    pub fn new(name: Arc<String>, namespace: Arc<String>) -> Self {
        AppKey {
            app_name: name,
            namespace,
        }
    }

    pub fn build_key(&self) -> String {
        format!("{}\x02{}", self.namespace, self.app_name)
    }
}

impl Ord for AppKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 先比较 namespace，再比较 name
        match self.namespace.cmp(&other.namespace) {
            std::cmp::Ordering::Equal => self.app_name.cmp(&other.app_name),
            other => other,
        }
    }
}

impl From<&str> for AppKey {
    fn from(value: &str) -> Self {
        let mut list = value.split('\x02');
        let namespace = list.next();
        let app_name = list.next();
        AppKey::new(
            Arc::new(app_name.unwrap_or("").to_string()),
            Arc::new(namespace.unwrap_or("").to_string()),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoDto {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub label: Arc<String>,
    pub register_type: String,
    pub instance_addrs: Option<Vec<Arc<String>>>,
}

impl AppInfoDto {
    pub fn new_from(app_info: &AppInfo, with_addrs: bool) -> Self {
        let instance_addrs = if with_addrs {
            let mut addrs = vec![];
            for (_, instance) in app_info.instance_map.iter() {
                addrs.push(instance.addr.clone());
            }
            Some(addrs)
        } else {
            None
        };
        AppInfoDto {
            app_name: app_info.app_name.clone(),
            namespace: app_info.namespace.clone(),
            label: app_info.label.clone(),
            register_type: app_info.register_type.to_str().to_owned(),
            instance_addrs,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppParam {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub label: Option<Arc<String>>,
    pub register_type: Option<RegisterType>,
    pub instance_addrs: Option<Vec<Arc<String>>>,
}

impl AppParam {
    pub fn build_app_key(&self) -> AppKey {
        AppKey::new(self.app_name.clone(), self.namespace.clone())
    }
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<AppManagerResult>")]
pub enum AppManagerReq {
    GetApp(AppKey),
    RegisterAppInstance(AppKey, Arc<String>),
    UnregisterAppInstance(AppKey, Arc<String>),
    GetAppInstanceAddrs(AppKey),
    QueryApp(AppQueryParam),
}

#[derive(Debug, Clone)]
pub enum AppManagerResult {
    None,
    AppInfo(Option<AppInfoDto>),
    InstanceAddrs(Arc<Vec<Arc<String>>>),
    AppPageInfo(usize, Vec<AppInfoDto>),
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "anyhow::Result<AppManagerRaftResult>")]
pub enum AppManagerRaftReq {
    UpdateApp(AppParam),
    RemoveApp(AppKey),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppManagerRaftResult {
    None,
}
