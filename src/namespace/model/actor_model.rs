use crate::namespace::model::namespace::{NamespaceInfo, NamespaceParam, NamespaceQueryParam};
use actix::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Message, Deserialize, Serialize)]
#[rtype(result = "anyhow::Result<NamespaceManagerRaftResult>")]
pub enum NamespaceManagerRaftReq {
    UpdateNamespace(NamespaceParam),
    Remove(Arc<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum NamespaceManagerRaftResult {
    None,
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<NamespaceManagerResult>")]
pub enum NamespaceManagerReq {
    GetNamespace(Arc<String>),
    SetWeak(Arc<String>),
    RemoveWeak(Arc<String>),
    QueryNamespace(NamespaceQueryParam),
    QueryList,
}

#[derive(Debug, Clone)]
pub enum NamespaceManagerResult {
    NamespaceInfo(Option<Arc<NamespaceInfo>>),
    NamespacePageInfo(usize, Vec<NamespaceInfo>),
    NamespaceList(Vec<NamespaceInfo>),
    None,
}
