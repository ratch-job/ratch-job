use std::sync::Arc;
use crate::webhook::model::{EventInfo, NotifyConfigModel, NotifyConfigModelOb, NotifyConfigPageQuery, NotifyConfigParam, NotifyEvent, NotifyObject, WebHookSource};
use actix::Message;
use serde::{Deserialize, Serialize};
use crate::job::model::job::{JobInfo, JobParam};
use crate::task::model::task::JobTaskInfo;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<WebhookManagerResult>")]
pub enum WebhookManagerReq {
    AddConfig(NotifyConfigModelOb),
    UpdateConfig(NotifyConfigModelOb),
    RemoveConfig(u64),
    QueryConfig(u64),
    QueryConfigPage(NotifyConfigPageQuery),
}

#[derive(Debug, Clone)]
pub enum WebhookManagerResult {
    None,
    ConfigInfo(Option<NotifyConfigModelOb>),
    ConfigPageInfo((usize, Vec<NotifyConfigModelOb>)),
}

#[derive(Debug, Clone, Message, Deserialize, Serialize)]
#[rtype(result = "anyhow::Result<WebhookManagerRaftResult>")]
pub enum WebhookManagerRaftReq {
    AddNotifyConfig(NotifyConfigParam),
    UpdateNotifyConfig(NotifyConfigParam),
    Remove(u64),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum WebhookManagerRaftResult {
    Info(Arc<NotifyConfigModelOb>),
    None,
}
