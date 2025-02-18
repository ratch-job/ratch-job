use crate::app::model::AppKey;
use crate::common::http_utils::HttpUtils;
use crate::job::core::JobManager;
use crate::task::request_client::XxlClient;
use crate::webhook::model::WebHookSource;
use actix::{Actor, Context};
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;

#[bean(inject)]
pub struct WebHookManager {
    webhook_object_map: HashMap<AppKey, Vec<WebHookObject>>,
    notify_event_map: HashMap<AppKey, HashMap<NotifyEvent, WebHookObject>>,
}

impl WebHookManager {
    pub fn new() -> Self {
        WebHookManager {
            webhook_object_map: Default::default(),
            notify_event_map: Default::default(),
        }
    }

    pub fn update_webhook_object(
        &mut self,
        app: AppKey,
        info: WebHookObject,
    ) -> anyhow::Result<()> {
        if let Some(vec) = self.webhook_object_map.get_mut(&app) {
            let find = vec
                .iter_mut()
                .find(|w| w.hook_source == info.hook_source.clone());
            if let Some(hook) = find {
                hook.token = info.token.clone();
            }
        } else {
            self.webhook_object_map.insert(app, vec![info]);
        }
        Ok(())
    }

    pub fn delete_webhook_object(
        &mut self,
        app: AppKey,
        source: WebHookSource,
    ) -> anyhow::Result<()> {
        if let Some(vec) = self.webhook_object_map.get_mut(&app) {
            let find = vec.iter_mut().position(|w| w.hook_source == source);
            if let Some(i) = find {
                vec.remove(i);
            }
        }
        Ok(())
    }

    pub fn update_event_map(
        &mut self,
        app: AppKey,
        event: NotifyEvent,
        object: WebHookObject,
    ) -> anyhow::Result<()> {
        if let Some(map) = self.notify_event_map.get_mut(&app) {
            map.insert(event, object);
        } else {
            let mut inner = HashMap::new();
            inner.insert(event, object);
            self.notify_event_map.insert(app, inner);
        }
        Ok(())
    }

    pub fn delete_event(&mut self, app: AppKey, event: NotifyEvent) -> anyhow::Result<()> {
        if let Some(map) = self.notify_event_map.get_mut(&app) {
            map.remove(&event);
        }
        Ok(())
    }

    pub async fn send_notify(&self, app: AppKey, event: NotifyEvent) {
        if let Some(map) = self.notify_event_map.get(&app) {
            if let Some(object) = map.get(&event) {
                match event {
                    NotifyEvent::ExeJobFail(fail_msg) => match &object.hook_source {
                        WebHookSource::WeiXin => {
                            let wx = WeChatChannel {
                                webhook_url: object.url.clone(),
                            };
                            let _ = wx.send(&fail_msg).await;
                        }
                        WebHookSource::DingDing => {}
                        WebHookSource::FeiShu => {}
                        WebHookSource::Jenkins => {}
                        WebHookSource::Other => {}
                    },
                };
            }
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum NotifyEvent {
    ExeJobFail(String),
}

trait NotificationChannel {
    async fn send(&self, message: &str) -> Result<(), String>;
}

struct WeChatChannel {
    webhook_url: Arc<String>,
}

impl NotificationChannel for WeChatChannel {
    async fn send(&self, message: &str) -> Result<(), String> {
        let payload = format!(
            r#"{{"msgtype": "text", "text": {{"content": "{}"}}}}"#,
            message
        );
        match HttpUtils::post_body(self.webhook_url.to_string(), payload, None, None).await {
            Ok(_) => {
                log::error!("WeChatChannel|send success!");
            }
            Err(err) => {
                log::error!("WeChatChannel|send error:{}", &err);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebHookObject {
    pub url: Arc<String>,
    pub hook_source: WebHookSource,
    pub token: Option<Arc<String>>,
}

impl Actor for WebHookManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebHookManager started");
    }
}

impl Inject for WebHookManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
    }
}
