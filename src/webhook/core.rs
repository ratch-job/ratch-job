use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use actix::{Actor, Context, Handler};
use anyhow::anyhow;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::app::model::AppKey;
use crate::common::http_utils::HttpUtils;
use crate::job::core::JobManager;
use crate::task::request_client::XxlClient;
use crate::webhook::actor_model::{WebhookManagerReq, WebhookManagerResult};
use crate::webhook::model::{EventInfo, NotifyEvent, WebHookObject, WebHookSource};

#[bean(inject)]
pub struct WebHookManager {
    webhook_object_map: HashMap<AppKey, Vec<WebHookObject>>,
    notify_event_map: HashMap<AppKey, Vec<EventInfo>>,
}

impl WebHookManager {
    pub fn new() -> Self {
        WebHookManager{
            webhook_object_map: Default::default(),
            notify_event_map: Default::default(),
        }
    }

    pub fn update_webhook_object(&mut self, app: AppKey, info: WebHookObject) -> anyhow::Result<()> {
        if let Some(vec) = self.webhook_object_map.get_mut(&app) {
            let find = vec.iter_mut().find(|w|w.hook_source == info.hook_source.clone());
            if let Some(hook) = find {
                hook.token = info.token.clone();
            }else{
                vec.push(info);
            }
        }else{
            self.webhook_object_map.insert(app, vec![info]);
        }
        Ok(())
    }

    pub fn delete_webhook_object(&mut self, app: AppKey, source: WebHookSource) -> anyhow::Result<()> {
        if let Some(vec) = self.webhook_object_map.get_mut(&app) {
            let find = vec.iter_mut().position(|w|w.hook_source == source);
            if let Some(i) = find {
                vec.remove(i);
            }
        }
        Ok(())
    }

    pub fn update_event_map(&mut self, app: AppKey, event: NotifyEvent, source: WebHookSource ) -> anyhow::Result<()> {
        if let Some(vec) = self.notify_event_map.get_mut(&app) {
            let find_opt = vec.iter_mut().find(|w|w.event == event&&w.source == source);
            if find_opt.is_some(){
                //已经存在了
                return Err(anyhow!("已经存在，不允许修改".to_string()))
            }else{
                vec.push(EventInfo{ event, source });
            }
        }else{
            self.notify_event_map.insert(app, vec![EventInfo{ event, source }]);
        }
        Ok(())
    }

    pub fn delete_event(&mut self, app: AppKey, event: NotifyEvent, source: WebHookSource) -> anyhow::Result<()> {
        if let Some(vec) = self.notify_event_map.get_mut(&app) {
            let find = vec.iter_mut().position(|w|w.event == event&&w.source == source);
            if let Some(i) = find {
                vec.remove(i);
            }
        }
        Ok(())
    }

    pub async fn send_notify(&self, app: AppKey, event: NotifyEvent) {
        if let Some(vec) = self.notify_event_map.get(&app) {
            for info in vec {
                if info.event == event.clone() {
                    match &event {
                        NotifyEvent::ExeJobFail(fail_msg) => {
                            let objects_opt = self.webhook_object_map.get(&app);
                            if let Some(obs) = objects_opt {
                                for ob_info in obs {
                                    match &ob_info.hook_source {
                                        WebHookSource::WeiXin => {
                                            let wx = WeChatChannel { webhook_url: ob_info.url.clone()};
                                            let _ = wx.send(&fail_msg.clone().unwrap_or_default()).await;
                                        }
                                        WebHookSource::DingDing => {
                                        }
                                        WebHookSource::FeiShu => {
                                        }
                                        WebHookSource::Jenkins => {
                                        }
                                        WebHookSource::Other => {
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn object_list(&self, app: AppKey) -> Vec<WebHookObject> {
        self.webhook_object_map.get(&app).map(|x|x.clone()).unwrap_or_default()
    }

    pub fn event_list(&self, app: AppKey) -> Vec<EventInfo> {
        self.notify_event_map.get(&app).map(|x|x.clone()).unwrap_or_default()
    }
}




trait NotificationChannel {
    async fn send(&self, message: &str) -> Result<(), String>;
}

struct WeChatChannel {
    webhook_url: Arc<String>,
}

impl NotificationChannel for WeChatChannel {
    async fn send(&self, message: &str) -> Result<(), String> {
        let payload = format!(r#"{{"msgtype": "text", "text": {{"content": "{}"}}}}"#, message);
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

impl Actor for WebHookManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebHookManager started");
    }
}

impl Handler<WebhookManagerReq> for WebHookManager {
    type Result = anyhow::Result<WebhookManagerResult>;

    fn handle(&mut self, msg: WebhookManagerReq, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            WebhookManagerReq::UpdateObject((app, object)) => {
                let _ = self.update_webhook_object(app, object);
            }
            WebhookManagerReq::RemoveObject((app, source)) => {
                let _ = self.delete_webhook_object(app, source);
            }
            WebhookManagerReq::UpdateEvent((app, event, source)) => {
                let _ = self.update_event_map(app, event, source);
            }
            WebhookManagerReq::RemoveEvent((app, event, source)) => {
                let _ = self.delete_event(app, event, source);
            }
            WebhookManagerReq::QueryObject((app, source)) => {}
            WebhookManagerReq::QueryEvent((app, event, object)) => {}
            WebhookManagerReq::QueryObjectPage(app) => {
                let vec = self.object_list(app);
                return Ok(WebhookManagerResult::ObjectPageInfo(vec.len(), vec))
            }
            WebhookManagerReq::QueryEventPage(app) => {
                let vec = self.event_list(app);
                return Ok(WebhookManagerResult::EventPageInfo(vec.len(), vec))
            }
        }
        Ok(WebhookManagerResult::None)
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