use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, WrapFuture};
use anyhow::anyhow;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::app::model::AppKey;
use crate::common::constant::SEQ_TASK_ID;
use crate::common::http_utils::HttpUtils;
use crate::job::core::JobManager;
use crate::sequence::{SequenceManager, SequenceRequest, SequenceResult};
use crate::task::request_client::XxlClient;
use crate::webhook::actor_model::{WebhookManagerReq, WebhookManagerResult};
use crate::webhook::model::{EventInfo, NotifyConfigModelOb, NotifyEvent, NotifyObject, WebHookSource};

#[bean(inject)]
pub struct WebHookManager {
    notify_object_map: HashMap<u64, NotifyConfigModelOb>,
    sequence_manager: Option<Addr<SequenceManager>>,
}

impl WebHookManager {
    pub fn new() -> Self {
        WebHookManager{
            notify_object_map: Default::default(),
            sequence_manager: None,
        }
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
            WebhookManagerReq::AddConfig(config_model) => {
                if self.notify_object_map.get(&config_model.id).is_some() {
                    return Err(anyhow!("添加配置失败，内部错误，相同配置的id已存在"));
                }
                self.notify_object_map.insert(config_model.id, config_model.clone());
                Ok(WebhookManagerResult::ConfigInfo(Some(config_model)))
            }
            WebhookManagerReq::UpdateConfig(model) => {
                if self.notify_object_map.get_mut(&model.id).is_some() {
                    self.notify_object_map.remove(&model.id);
                    self.notify_object_map.insert(model.id, model);
                }
               Ok(WebhookManagerResult::None)
            }
            WebhookManagerReq::RemoveConfig(id) => {
                self.notify_object_map.remove(&id);
                Ok(WebhookManagerResult::None)
            }
            WebhookManagerReq::QueryConfig(id) => {
                Ok(WebhookManagerResult::ConfigInfo(self.notify_object_map.get(&id).map(|x|x.clone())))
            }
            WebhookManagerReq::QueryConfigPage(_) => {
                let vec = self.notify_object_map.values().map(|x|x.clone()).collect::<Vec<_>>();
                Ok(WebhookManagerResult::ConfigPageInfo((vec.len(), vec)))
            }
        }
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
        self.sequence_manager = factory_data.get_actor();
    }
}