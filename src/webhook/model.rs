use std::str::FromStr;
use std::sync::Arc;
use anyhow::{anyhow};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};
use crate::app::model::{AppKey};
use crate::common::pb::data_object::NotifyConfigDo;
use crate::webhook::model::ChannelType::{Email, WebHook};

struct WebHookData {
    source: WebHookSource,
    token: Option<String>,
    url: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotifyConfigPageQuery {
    pub app_key: AppKey,
}


///对象化的配置实体
/// #[derive(Debug)]
#[derive(Debug, Clone, Deserialize, Serialize,)]
pub struct NotifyConfigModelOb {
    pub id: u64,
    pub model: NotifyConfigModel
}

impl NotifyConfigModelOb {
    pub fn to_do(&self) -> NotifyConfigDo{
        NotifyConfigDo{
            id: self.id,
            enable: self.model.enable,
            app_name: Default::default(),
            namespace: Default::default(),
            name: Default::default(),
            channel_type: Default::default(),
            channel_sub_type: Default::default(),
            url: Default::default(),
            email: Default::default(),
            username: Default::default(),
            password: Default::default(),
            token: Default::default(),
            version_id: 0,
            last_modified_millis: 0,
            create_time: 0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotifyConfigModel {
    pub enable: bool,
    pub app_key: AppKey,
    pub name: String,
    pub channel_type: ChannelType,
    pub channel_config: ChannelConfig,
}

impl TryFrom<NotifyConfigParam> for NotifyConfigModel {
    type Error = anyhow::Error;
    fn try_from(value: NotifyConfigParam) -> Result<NotifyConfigModel, anyhow::Error> {
        let channel = ChannelType::from_str(value.channel_type.unwrap_or_default().as_str())?;
        let ((chan, cfg)) = match channel {
            WebHook(_) => {
                let source = WebHookSource::from_str(value.channel_sub_type.unwrap_or_default().as_str())?;
                let c = match source {
                    WebHookSource::FeiShu => {
                        let config = HookConfig{
                            url: value.url.unwrap_or_default().to_string(),
                            password: value.password.unwrap_or_default().to_string(),
                            r#type: WebHookSource::FeiShu.to_string()
                        };
                        ChannelConfig::Webhook(config)
                    }
                };
                (WebHook(source), c)
            }
            Email(_) => {
                let source = EmailType::from_str(value.channel_sub_type.unwrap_or_default().as_str())?;
                let e = match source {
                    EmailType::Common => {
                        ChannelConfig::Email(EmailConfig{
                            url: value.url.unwrap_or_default().to_string(),
                            email_addr: value.email.unwrap_or_default().to_string(),
                            username: value.username.unwrap_or_default().to_string(),
                            password: value.password.unwrap_or_default().to_string(),
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
        Ok(NotifyConfigModel {
            enable: value.enable,
            app_key: AppKey{
                app_name: value.app_name.unwrap_or_default(),
                namespace: value.namespace.unwrap_or_default(),
            },
            name: value.name.unwrap_or_default().to_string(),
            channel_type: chan,
            channel_config: cfg,
        })
    }
}

impl<'a> From<NotifyConfigDo<'a>> for NotifyConfigModelOb {
    fn from(value: NotifyConfigDo<'a>) -> Self {
        let channel = ChannelType::from_str(value.channel_type.as_ref()).unwrap();
        let ((chan, cfg)) = match channel {
            WebHook(_) => {
                let source = WebHookSource::from_str(value.channel_sub_type.as_ref()).unwrap();
                let c = match source {
                    WebHookSource::FeiShu => {
                        let config = HookConfig{
                            url: value.url.to_string(),
                            password: value.password.to_string(),
                            r#type: WebHookSource::FeiShu.to_string()
                        };
                        ChannelConfig::Webhook(config)
                    }
                };
                (WebHook(source), c)
            }
            Email(_) => {
                let source = EmailType::from_str(value.channel_sub_type.as_ref()).unwrap();
                let e = match source {
                    EmailType::Common => {
                        ChannelConfig::Email(EmailConfig{
                            url: value.url.to_string(),
                            email_addr: value.email.to_string(),
                            username: value.username.to_string(),
                            password: value.password.to_string(),
                            r#type: format!("{}", source)
                        })
                    }
                };
                (Email(source), e)
            }
            ChannelType::None => {
                panic!("to NotifyConfigModelOb fail!")
            }
        };
        Self{
            id: value.id,
            model: NotifyConfigModel {
                enable: value.enable,
                app_key: AppKey{
                    app_name: Arc::new(value.app_name.to_string()),
                    namespace: Arc::new(value.namespace.to_string()),
                },
                name: value.name.to_string(),
                channel_type: chan,
                channel_config: cfg,
            },
        }
    }
}

#[derive(Debug, Clone, EnumString, Deserialize, Serialize,EnumIter, strum_macros::Display, Default)]
pub enum ChannelType {
    #[default]
    None,
    WebHook(WebHookSource) ,
    Email(EmailType),
}

#[derive(Debug, Clone, EnumString, Deserialize, Serialize, EnumIter, strum_macros::Display, Default)]
pub enum EmailType {
    #[default]
    Common,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ChannelConfig {
    Email(EmailConfig),
    Webhook(HookConfig)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub r#type: String,
    pub url: String,
    pub email_addr: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookConfig {
    pub r#type: String,
    pub url: String,
    pub password: String,
}

#[derive(Debug, Clone,Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AppNotifyEvent {
    ExeJobFail
}

#[derive(Clone, Debug, Eq, EnumString, Serialize, Deserialize, PartialEq, EnumIter, strum_macros::Display, Default)]
pub enum WebHookSource {
    #[default]
    FeiShu,
}

#[derive(Debug, Clone,Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum NotifyEvent {
    ExeJobFail(Option<String>)
}

#[derive(Debug, Clone,Eq, PartialEq, Serialize, Deserialize)]
pub struct EventInfo {
    pub id: u32,
    pub event: String,
    // 短信， 邮件， webhook
    pub channel: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NotifyObject {
    pub id: u32,
    pub url: Arc<String>,
    pub hook_source: WebHookSource,
    pub token: Option<Arc<String>>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyConfigParam {
    pub id: Option<u64>,
    pub enable: bool,
    pub app_name: Option<Arc<String>>,
    pub namespace: Option<Arc<String>>,
    pub name: Option<Arc<String>>,
    pub channel_type: Option<Arc<String>>,
    pub channel_sub_type: Option<Arc<String>>,
    pub url: Option<Arc<String>>,
    pub email: Option<Arc<String>>,
    pub username: Option<Arc<String>>,
    pub password: Option<Arc<String>>,
    pub token: Option<Arc<String>>,
}

/*///配置表字段
///扁平化的配置实体
struct NotifyConfigTable {
    id: u32,
    pub enable: bool,
    pub app_name: Arc<String>,
    pub namespace: Arc<String>,
    name: String, //配置的名称
    channel_type: String,//webhook, email
    channel_sub_type: String,//email-网易/腾讯   webhook-企业微信群机器人/飞书
    url: String, //webhook url或者email stmp地址
    email: Option<String>,
    username: String,
    password: String,
    token: Option<String>,
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub create_time: u64,
}*/
