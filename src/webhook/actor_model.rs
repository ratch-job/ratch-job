use crate::webhook::model::{EventInfo, NotifyConfigModel, NotifyConfigModelOb, NotifyConfigPageQuery, NotifyEvent, NotifyObject, WebHookSource};
use actix::Message;
use crate::console::model::webhook_model;

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
