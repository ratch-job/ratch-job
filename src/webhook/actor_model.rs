use crate::app::model::AppKey;
use crate::webhook::model::{EventInfo, NotifyEvent, WebHookObject, WebHookSource};
use actix::Message;

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<WebhookManagerResult>")]
pub enum WebhookManagerReq {
    UpdateObject((AppKey, WebHookObject)),
    RemoveObject((AppKey, WebHookSource)),
    UpdateEvent((AppKey, NotifyEvent, WebHookSource)),
    RemoveEvent((AppKey, NotifyEvent, WebHookSource)),
    QueryObject((AppKey, WebHookSource)),
    QueryEvent((AppKey, NotifyEvent, WebHookSource)),
    QueryObjectPage(AppKey),
    QueryEventPage(AppKey),
}

#[derive(Debug, Clone)]
pub enum WebhookManagerResult {
    None,
    ObjectInfo(Option<WebHookObject>),
    EventInfo(Option<EventInfo>),
    ObjectPageInfo(usize, Vec<WebHookObject>),
    EventPageInfo(usize, Vec<EventInfo>),
}