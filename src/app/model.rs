use crate::common::datetime_utils::now_millis;
use actix::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RegisterType {
    Auto,
    Manual,
}

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: Arc<String>,
    pub namespace: Arc<String>,
    pub label: Arc<String>,
    pub register_type: RegisterType,
    pub instance_map: HashMap<Arc<String>, AppInstance>,
}

impl AppInfo {
    pub fn new(
        name: Arc<String>,
        namespace: Arc<String>,
        label: Arc<String>,
        register_type: RegisterType,
    ) -> Self {
        AppInfo {
            name,
            namespace,
            label,
            register_type,
            instance_map: HashMap::new(),
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

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct AppKey {
    pub name: Arc<String>,
    pub namespace: Arc<String>,
}

impl AppKey {
    pub fn new(name: Arc<String>, namespace: Arc<String>) -> Self {
        AppKey { name, namespace }
    }
}

impl Ord for AppKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 先比较 namespace，再比较 name
        match self.namespace.cmp(&other.namespace) {
            std::cmp::Ordering::Equal => self.name.cmp(&other.name),
            other => other,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppParam {
    pub name: Arc<String>,
    pub namespace: Arc<String>,
    pub label: Option<Arc<String>>,
    pub register_type: Option<RegisterType>,
    pub instance_addrs: Option<Vec<Arc<String>>>,
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<AppManagerResult>")]
pub enum AppManagerReq {
    UpdateApp(AppParam),
    RemoveApp(AppKey),
    RegisterAppInstance(AppKey, Arc<String>),
    UnregisterAppInstance(AppKey, Arc<String>),
}

#[derive(Debug, Clone)]
pub enum AppManagerResult {
    None,
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "anyhow::Result<AppManagerRaftResult>")]
pub enum AppManagerRaftReq {
    UpdateApp(AppParam),
    RemoveApp(AppParam),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppManagerRaftResult {
    None,
}
