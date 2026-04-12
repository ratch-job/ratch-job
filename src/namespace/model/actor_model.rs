use crate::namespace::model::namespace::{NamespaceInfo, NamespaceParam, NamespaceQueryParam};
use actix::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Message, Deserialize, Serialize)]
#[rtype(result = "anyhow::Result<NamespaceManagerRaftResult>")]
pub enum NamespaceManagerRaftReq {
    AddNamespace(NamespaceParam),
    UpdateNamespace(NamespaceParam),
    Remove(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum NamespaceManagerRaftResult {
    NamespaceInfo(Arc<NamespaceInfo>),
    None,
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<NamespaceManagerResult>")]
pub enum NamespaceManagerReq {
    GetNamespace(String),
    QueryNamespace(NamespaceQueryParam),
}

#[derive(Debug, Clone)]
pub enum NamespaceManagerResult {
    NamespaceInfo(Option<Arc<NamespaceInfo>>),
    NamespacePageInfo(usize, Vec<NamespaceInfo>),
    None,
}
