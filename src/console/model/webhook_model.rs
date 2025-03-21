use crate::app::model::AppKey;
use crate::webhook::model::ChannelType::{Email, WebHook};
use crate::webhook::model::{ChannelConfig, ChannelType, EmailConfig, EmailType, HookConfig, NotifyConfigModel, NotifyConfigModelOb, NotifyConfigPageQuery, WebHookSource};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NotifyConfigAdd {
    pub enable: bool,
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
                            r#type: WebHookSource::FeiShu.to_string()
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
                            r#type: format!("{}", source)
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
            enable: self.enable,
            app_key: AppKey {
                app_name: self.app_name.clone(),
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
    pub enable: bool,
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
                            r#type: source.to_string(),
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
                            r#type: source.to_string(),
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
            enable: self.enable,
            app_key: AppKey {
                app_name: self.app_name.clone(),
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
    // pub name: Option<String>, //配置的名称
}

impl NotifyConfigQuery {
    pub(crate) fn to_param(&self) -> NotifyConfigPageQuery {
        let query = NotifyConfigPageQuery{
            app_key: AppKey{ app_name: self.app_name.clone(), namespace: self.namespace.clone() },
        };
        query
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotifyConfigRemove {
    pub id: u64
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotifyConfigInfo {
    pub id: u64
}