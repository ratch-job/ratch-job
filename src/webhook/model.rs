use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq)]
pub enum WebHookSource {
    WeiXin,
    DingDing,
    FeiShu,
    Jenkins,
    Other,
}

impl WebHookSource {
    pub fn from_str(s: &str) -> WebHookSource {
        match s {
            "WEIXIN" => WebHookSource::WeiXin,
            "DINGDING" => WebHookSource::DingDing,
            "FEISHU" => WebHookSource::FeiShu,
            "JENKINS" => WebHookSource::Jenkins,
            _ => WebHookSource::Other,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            WebHookSource::WeiXin => "WEIXIN",
            WebHookSource::DingDing => "DINGDING",
            WebHookSource::FeiShu => "FEISHU",
            WebHookSource::Jenkins => "JENKINS",
            WebHookSource::Other => "",
        }
    }
}

#[derive(Debug, Clone,Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum NotifyEvent {
    ExeJobFail(Option<String>)
}
impl NotifyEvent {
    pub fn from_str(event_type: &str) -> NotifyEvent {
        NotifyEvent::ExeJobFail(None)
    }

    pub fn to_str(&self) -> &str {
        "ExeJobFail"
    }

    pub fn from_type_message(_event_type: String, msg: Option<String>) -> NotifyEvent {
        NotifyEvent::ExeJobFail(msg)
    }
}



#[derive(Debug, Clone,Eq, PartialEq, Serialize, Deserialize)]
pub struct EventInfo {
    pub event: NotifyEvent,
    pub source: WebHookSource,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WebHookObject {
    pub url: Arc<String>,
    pub hook_source: WebHookSource,
    pub token: Option<Arc<String>>
}
