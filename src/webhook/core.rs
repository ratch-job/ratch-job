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
use quick_protobuf::{BytesReader, Writer};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::app::model::AppKey;
use crate::common::byte_utils::id_to_bin;
use crate::common::constant::{JOB_NOTIFY_NOTIFY_TABLE_NAME, JOB_TABLE_NAME, JOB_TASK_TABLE_NAME, SEQ_TASK_ID};
use crate::common::http_utils::HttpUtils;
use crate::common::pb::data_object::{JobDo, JobTaskDo, NotifyConfigDo};
use crate::job::core::JobManager;
use crate::job::model::actor_model::{JobManagerRaftReq, JobManagerRaftResult};
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::{RaftApplyDataRequest, RaftApplyDataResponse};
use crate::raft::store::raftsnapshot::{SnapshotWriterActor, SnapshotWriterRequest};
use crate::sequence::{SequenceManager, SequenceRequest, SequenceResult};
use crate::task::model::task::JobTaskInfo;
use crate::task::request_client::XxlClient;
use crate::webhook::actor_model::{WebhookManagerRaftReq, WebhookManagerRaftResult, WebhookManagerReq, WebhookManagerResult};
use crate::webhook::model::{EmailConfig, EmailType, EventInfo, HookConfig, NotifyConfigModel, NotifyConfigModelOb, NotifyEvent, NotifyObject, WebHookSource};

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

    pub fn add_notify_config(&mut self, oj: NotifyConfigModelOb) -> Result<Arc<NotifyConfigModelOb>> {
        if self.notify_object_map.get(&oj.id).is_some() {
            return Err(anyhow!("添加配置失败，内部错误，相同配置的id已存在"));
        }
        self.notify_object_map.insert(oj.id, oj.clone());
        Ok(Arc::new(oj))
    }

    pub fn update_notify_config(&mut self, oj: NotifyConfigModelOb) -> Result<()> {
        if self.notify_object_map.get(&oj.id).is_none() {
            return Err(anyhow!("配置不存在，不能修改"));
        }
        if self.notify_object_map.get_mut(&oj.id).is_some() {
            self.notify_object_map.remove(&oj.id);
            self.notify_object_map.insert(oj.id, oj);
        }
        Ok(())
    }

    pub fn remove_notify_config(&mut self, id: u64) -> Result<u64> {
        self.notify_object_map.remove(&id);
        Ok(id)
    }

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> Result<()> {
        for (key, ob) in &self.notify_object_map {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf);
                let value_do = ob.to_do();
                writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: JOB_TABLE_NAME.clone(),
                key: id_to_bin(*key),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        Ok(())
    }

    fn load_snapshot_record(&mut self, record: SnapshotRecordDto) -> Result<()> {
        if record.tree.as_str() == JOB_NOTIFY_NOTIFY_TABLE_NAME.as_str() {
            let mut reader = BytesReader::from_bytes(&record.value);
            let value_do: NotifyConfigDo = reader.read_message(&record.value)?;
            let _ = self.update_notify_config(value_do.into());
        }
        Ok(())
    }

    fn load_completed(&mut self, _ctx: &mut Context<Self>) -> anyhow::Result<()> {
        Ok(())
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

impl Handler<WebhookManagerRaftReq> for WebHookManager {
    type Result = Result<WebhookManagerRaftResult>;

    fn handle(&mut self, msg: WebhookManagerRaftReq, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            WebhookManagerRaftReq::AddNotifyConfig(add) => {
                let id_opt = add.id;
                let mut model = NotifyConfigModel::try_from(add)?;
                if let Some(tid) = id_opt {
                    let ob = NotifyConfigModelOb{
                        id: tid,
                        model,
                    };
                    return Ok(WebhookManagerRaftResult::Info(self.add_notify_config(ob)?));
                }
            }
            WebhookManagerRaftReq::UpdateNotifyConfig(update) => {
                let id_opt = update.id;
                let model = NotifyConfigModel::try_from(update)?;
                if let Some(tid) = id_opt {
                    let ob = NotifyConfigModelOb {
                        id: tid,
                        model,
                    };
                    self.update_notify_config(ob)?;
                }
            }
            WebhookManagerRaftReq::Remove(id) => {
                self.remove_notify_config(id)?;
            }
        }
        Ok(WebhookManagerRaftResult::None)
    }
}

impl Handler<RaftApplyDataRequest> for WebHookManager {
    type Result = Result<RaftApplyDataResponse>;

    fn handle(&mut self, msg: RaftApplyDataRequest, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            RaftApplyDataRequest::BuildSnapshot(writer) => {
                self.build_snapshot(writer)?;
            }
            RaftApplyDataRequest::LoadSnapshotRecord(record) => {
                self.load_snapshot_record(record)?;
            }
            RaftApplyDataRequest::LoadCompleted => {
                self.load_completed(ctx)?;
            }
        }
        Ok(RaftApplyDataResponse::None)
    }
}