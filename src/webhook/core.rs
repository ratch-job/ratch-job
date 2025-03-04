use std::cmp::PartialEq;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, WrapFuture};
use anyhow::{anyhow, Result};
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::{From, Headers, To};
use lettre::message::{header, Mailbox, Mailboxes};
use lettre::transport::smtp::authentication::Credentials;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::app::model::AppKey;
use crate::common::constant::SEQ_TASK_ID;
use crate::common::http_utils::HttpUtils;
use crate::job::core::JobManager;
use crate::sequence::{SequenceManager, SequenceRequest, SequenceResult};
use crate::task::request_client::XxlClient;
use crate::webhook::actor_model::{WebhookManagerReq, WebhookManagerResult};
use crate::webhook::model::{EmailConfig, EmailType, EventInfo, HookConfig, NotifyConfigModelOb, NotifyEvent, NotifyObject, WebHookSource};

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
    async fn send(&self, title: &str, message: &str, received: Vec<String>) -> Result<()>;
}

impl NotificationChannel for HookConfig {
    async fn send(&self, title: &str, message: &str, received: Vec<String>) -> Result<()> {
        let source = WebHookSource::from_str(self.r#type.as_str()).map_err(|e|anyhow!(e))?;
        match source {
            WebHookSource::FeiShu => {
                let payload = format!(r#"{{"msg_type": "text", "content": {{"text": "{}"}}}}"#, message);
                match HttpUtils::post_body(self.url.to_string(), payload, None, None).await {
                    Ok(_) => {
                        log::error!("feishu|send success!");
                    }
                    Err(err) => {
                        log::error!("feishu|send error:{}", &err);
                    }
                }
            }
        }
        Ok(())
    }
}

impl NotificationChannel for EmailConfig {
    async fn send(&self, title: &str, message: &str, received: Vec<String>) -> Result<()> {
        let email = EmailType::from_str(self.r#type.as_str()).map_err(|e|anyhow!(e))?;
        match email {
            EmailType::Common => {
                let tos = received.into_iter().filter_map(|r|r.parse::<Mailbox>().ok()).collect::<Vec<_>>();
                let mut email_builder = Message::builder()
                    .from(self.email_addr.parse()?);
                for b in tos {
                    email_builder = email_builder.to(b);
                }
                let message = email_builder.subject(title)
                    .body(message.to_string())?;
                let creds = Credentials::new(self.email_addr.clone(), self.password.to_string());
                let mailer = SmtpTransport::relay(&self.url)?
                    .credentials(creds)
                    .build();
                // 发送邮件
                mailer.send(&message)?;
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