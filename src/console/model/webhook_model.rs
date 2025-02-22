use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::app::model::AppKey;
use crate::webhook::model::{NotifyEvent, WebHookObject, WebHookSource};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectQueryListRequest {
    app_name: String,
    namespace: String,
}

impl ObjectQueryListRequest {
    pub fn to_param(&self) -> AppKey {
        AppKey {
            name: Arc::new(self.app_name.clone()),
            namespace: Arc::new(self.namespace.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectUpdateRequest {
    app_name: String,
    namespace: String,
    source: String,
    url: String,
    token: Option<String>
}

impl ObjectUpdateRequest {
    pub fn to_param(&self) -> (AppKey, WebHookObject) {
        (AppKey {
            name: Arc::new(self.app_name.clone()),
            namespace: Arc::new(self.namespace.clone()),
        }, WebHookObject{
            url: Arc::new(self.url.clone()),
            hook_source: WebHookSource::from_str(&self.source),
            token: self.token.clone().map(|x|Arc::new(x)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectRemoveRequest {
    app_name: String,
    namespace: String,
    source: String,
}

impl ObjectRemoveRequest {
    pub fn to_param(&self) -> (AppKey, WebHookSource) {
        (AppKey{ name: Arc::new(self.app_name.clone()), namespace: Arc::new(self.namespace.clone()) }, WebHookSource::from_str(&self.source))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventQueryListRequest {
    app_name: String,
    namespace: String,
}

impl EventQueryListRequest {
    pub fn to_param(&self) -> AppKey {
        AppKey {
            name: Arc::new(self.app_name.clone()),
            namespace: Arc::new(self.namespace.clone()),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventRemoveRequest {
    app_name: String,
    namespace: String,
    source: String,
    event_type: String,
}

impl EventRemoveRequest {
    pub fn to_param(&self) -> (AppKey, NotifyEvent, WebHookSource) {
        (AppKey {
            name: Arc::new(self.app_name.clone()),
            namespace: Arc::new(self.namespace.clone()),
        }, NotifyEvent::from_str(&self.event_type), WebHookSource::from_str(&self.source))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventUpdateRequest {
    app_name: String,
    namespace: String,
    source: String,
    event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    event_type: String,
    message: Option<String>,
}

impl EventUpdateRequest {
    pub fn to_param(&self) -> (AppKey, NotifyEvent, WebHookSource) {
        (AppKey {
            name: Arc::new(self.app_name.clone()),
            namespace: Arc::new(self.namespace.clone()),
        }, NotifyEvent::from_type_message(self.event.event_type.clone(), self.event.message.clone()), WebHookSource::from_str(&self.source))
    }
}