use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use crate::app::model::AppKey;
use crate::webhook::model::{ChannelConfig, ChannelType, EmailConfig, EmailType, HookConfig, NotifyConfigModel, NotifyConfigModelOb, NotifyConfigPageQuery, NotifyEvent, NotifyObject, WebHookSource};
use crate::webhook::model::ChannelType::{Email, WebHook};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotifyConfigAdd {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub name: String, //配置的名称
    pub channel_type: String,//webhook, email
    pub channel_sub_type: String,//email-网易/腾讯   webhook-企业微信群机器人/飞书
    pub url: String, //webhook url或者email stmp地址
    pub email: String,
    pub username: String,
    pub password: String,
    token: Option<String>,
}

impl NotifyConfigAdd {
    pub(crate) fn to_param(&self) -> anyhow::Result<NotifyConfigModel> {
        let channel = ChannelType::from_str(&self.channel_type.clone())?;
        let ((chan, cfg)) = match channel {
            WebHook(_) => {
                let source = WebHookSource::from_str(&self.channel_sub_type)?;
                let c = match source {
                    WebHookSource::FeiShu => {
                        let config = HookConfig{
                            url: self.url.clone(),
                            password: self.password.clone(),
                        };
                        ChannelConfig::Webhook(config)
                    }
                };
                (WebHook(source), c)
            }
            Email(_) => {
                let source = EmailType::from_str(&self.channel_sub_type)?;
                let e = match source {
                    EmailType::Common => {
                        ChannelConfig::Email(EmailConfig{
                            url: self.url.clone(),
                            email_addr: self.email.clone(),
                            username: self.username.clone(),
                            password: self.password.clone(),
                        })
                    }
                };
                (Email(source), e)
            }
            ChannelType::None => {
                return Err(anyhow!("ChannelType error!"));
            }
        };
        let model = NotifyConfigModel{
            app_key: AppKey {
                name: self.app_name.clone(),
                namespace: self.namespace.clone(),
            },
            name: self.name.clone(),
            channel_type: chan,
            channel_config: cfg,
        };
        Ok(model)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotifyConfigUpdate {
    pub id: u64,
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub name: String, //配置的名称
    pub channel_type: String,//webhook, email
    pub channel_sub_type: String,//email-网易/腾讯   webhook-企业微信群机器人/飞书
    pub url: String, //webhook url或者email stmp地址
    pub email: String,
    pub username: String,
    pub password: String,
    token: Option<String>,
}

impl NotifyConfigUpdate {
    pub(crate) fn to_param(&self) -> anyhow::Result<NotifyConfigModelOb> {
        let channel = ChannelType::from_str(&self.channel_type.clone())?;
        let ((chan, cfg)) = match channel {
            WebHook(_) => {
                let source = WebHookSource::from_str(&self.channel_sub_type)?;
                let c = match source {
                    WebHookSource::FeiShu => {
                        let config = HookConfig{
                            url: self.url.clone(),
                            password: self.password.clone(),
                        };
                        ChannelConfig::Webhook(config)
                    }
                };
                (WebHook(source), c)
            }
            Email(_) => {
                let source = EmailType::from_str(&self.channel_sub_type)?;
                let e = match source {
                    EmailType::Common => {
                        ChannelConfig::Email(EmailConfig{
                            url: self.url.clone(),
                            email_addr: self.email.clone(),
                            username: self.username.clone(),
                            password: self.password.clone(),
                        })
                    }
                };
                (Email(source), e)
            }
            ChannelType::None => {
                return Err(anyhow!("ChannelType error!"));
            }
        };
        let model = NotifyConfigModel{
            app_key: AppKey {
                name: self.app_name.clone(),
                namespace: self.namespace.clone(),
            },
            name: self.name.clone(),
            channel_type: chan,
            channel_config: cfg,
        };
        Ok(NotifyConfigModelOb{ id: self.id, model })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotifyConfigQuery {
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    pub name: String, //配置的名称
}

impl NotifyConfigQuery {
    pub(crate) fn to_param(&self) -> NotifyConfigPageQuery {
        let query = NotifyConfigPageQuery{
            app_key: AppKey{ name: self.app_name.clone(), namespace: self.namespace.clone() },
            name: self.name.clone(),
        };
        query
    }
}



/*#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub fn to_param(&self) -> (AppKey, NotifyObject) {
        (AppKey {
            name: Arc::new(self.app_name.clone()),
            namespace: Arc::new(self.namespace.clone()),
        }, NotifyObject {
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
}*/