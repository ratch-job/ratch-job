use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq)]
pub enum WebHookSource {
    WeiXin,
    DingDing,
    FeiShu,
    Jenkins,
    Other
}

impl WebHookSource {
    pub fn from_str(s: &str) -> WebHookSource {
        match s {
            "WEIXIN" => WebHookSource::WeiXin,
            "DINGDING" => WebHookSource::DingDing,
            "FEISHU" => WebHookSource::FeiShu,
            "JENKINS" => WebHookSource::Jenkins,
            _ => WebHookSource::Other,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            WebHookSource::WeiXin => "WEIXIN",
            WebHookSource::DingDing => "DINGDING",
            WebHookSource::FeiShu => "FEISHU",
            WebHookSource::Jenkins => "JENKINS",
            WebHookSource::Other => "",
        }
    }
}