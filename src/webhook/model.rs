use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};
use crate::app::model::AppKey;

struct WebHookData {
    source: WebHookSource,
    token: Option<String>,
    url: String
}

/*///配置表字段
///扁平化的配置实体
struct NotifyConfigTable {
    id: u32,
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
    create_at: DateTime<Local>,
    update_at: DateTime<Local>,
}*/

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotifyConfigModel {
    pub app_key: AppKey,
    pub name: String,
    pub channel_type: ChannelType,
    pub channel_config: ChannelConfig,
}

#[derive(Debug, Clone, EnumString, Deserialize, Serialize, EnumIter, strum_macros::Display, Default)]
pub enum ChannelType {
    WebHook(WebHookSource),
    Email(EmailType),
    #[default]
    None
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
