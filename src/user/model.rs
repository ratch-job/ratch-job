use crate::common::model::privilege::PrivilegeGroup;
use crate::common::pb::data_object::UserInfoDo;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub username: Arc<String>,
    pub nickname: String,
    pub password: String,
    pub password_hash: String,
    pub gmt_create: i64,
    pub gmt_modified: i64,
    pub enable: bool,
    pub roles: Vec<Arc<String>>,
    pub extend_info: HashMap<String, String>,
    pub namespace_privilege: PrivilegeGroup<Arc<String>>,
    pub app_privilege: PrivilegeGroup<Arc<String>>,
}

impl UserInfo {
    pub fn to_do(&self) -> UserInfoDo {
        UserInfoDo {
            username: Cow::Borrowed(&self.username),
            nickname: Cow::Borrowed(&self.nickname),
            password: Cow::Borrowed(&self.password),
            password_hash: Cow::Borrowed(&self.password_hash),
            gmt_create: self.gmt_create,
            gmt_modified: self.gmt_modified,
            enable: self.enable,
            roles: self
                .roles
                .iter()
                .map(|r| Cow::Borrowed(r.as_str()))
                .collect(),
            extend_info: self
                .extend_info
                .iter()
                .map(|(k, v)| (Cow::Borrowed(k.as_str()), Cow::Borrowed(v.as_str())))
                .collect(),
            namespace_privilege: Some(self.namespace_privilege.to_do()),
            app_privilege: Some(self.app_privilege.to_do()),
        }
    }
}

impl<'a> From<UserInfoDo<'a>> for UserInfo {
    fn from(record: UserInfoDo) -> Self {
        UserInfo {
            username: Arc::new(record.username.to_string()),
            nickname: record.nickname.to_string(),
            password: record.password.to_string(),
            password_hash: record.password_hash.to_string(),
            gmt_create: record.gmt_create,
            gmt_modified: record.gmt_modified,
            enable: record.enable,
            roles: record
                .roles
                .into_iter()
                .map(|e| Arc::new(e.to_string()))
                .collect(),
            extend_info: record
                .extend_info
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            namespace_privilege: record
                .namespace_privilege
                .map(|pg| pg.into())
                .unwrap_or_default(),
            app_privilege: record.app_privilege.map(|pg| pg.into()).unwrap_or_default(),
        }
    }
}
