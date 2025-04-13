use crate::common::model::privilege::{PrivilegeGroup, PrivilegeGroupOptionParam};
use crate::common::pb::data_object::UserInfoDo;
use crate::user::build_password_hash;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub username: Arc<String>,
    pub nickname: String,
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
    pub fn update(&mut self, record: UserDto) {
        if let Some(nickname) = record.nickname {
            self.nickname = nickname;
        }
        if let Some(password) = record.password {
            self.password_hash = build_password_hash(&password).unwrap_or_default();
        }
        if let Some(password_hash) = record.password_hash {
            self.password_hash = password_hash;
        }
        if let Some(gmt_create) = record.gmt_create {
            self.gmt_create = gmt_create;
        }
        if let Some(gmt_modified) = record.gmt_modified {
            self.gmt_modified = gmt_modified;
        }
        if let Some(enable) = record.enable {
            self.enable = enable;
        }
        if let Some(roles) = record.roles {
            self.roles = roles;
        }
        if let Some(extend_info) = record.extend_info {
            self.extend_info = extend_info;
        }
        if let Some(namespace_privilege) = record.namespace_privilege {
            self.namespace_privilege.update(namespace_privilege);
        }
        if let Some(app_privilege) = record.app_privilege {
            self.app_privilege.update(app_privilege);
        }
    }

    pub fn to_do(&self) -> UserInfoDo {
        UserInfoDo {
            username: Cow::Borrowed(&self.username),
            nickname: Cow::Borrowed(&self.nickname),
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

/// 用户信息对象，用于参数传输
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub username: Arc<String>,
    pub nickname: Option<String>,
    pub password: Option<String>,
    pub password_hash: Option<String>,
    pub gmt_create: Option<i64>,
    pub gmt_modified: Option<i64>,
    pub enable: Option<bool>,
    pub roles: Option<Vec<Arc<String>>>,
    pub extend_info: Option<HashMap<String, String>>,
    pub namespace_privilege: Option<PrivilegeGroupOptionParam<Arc<String>>>,
    pub app_privilege: Option<PrivilegeGroupOptionParam<Arc<String>>>,
}

impl From<UserDto> for UserInfo {
    fn from(record: UserDto) -> Self {
        let password_hash = if record.password_hash.is_none() {
            if let Some(p) = record.password.as_ref() {
                build_password_hash(p).unwrap_or_default()
            } else {
                "".to_string()
            }
        } else {
            record.password_hash.unwrap_or_default()
        };
        let mut namespace_privilege = PrivilegeGroup::all();
        namespace_privilege.update_option(record.namespace_privilege);
        let mut app_privilege = PrivilegeGroup::all();
        app_privilege.update_option(record.app_privilege);
        UserInfo {
            username: record.username,
            nickname: record.nickname.unwrap_or_default(),
            password_hash,
            gmt_create: record.gmt_create.unwrap_or_default(),
            gmt_modified: record.gmt_modified.unwrap_or_default(),
            enable: record.enable.unwrap_or_default(),
            roles: record.roles.unwrap_or_default(),
            extend_info: record.extend_info.unwrap_or_default(),
            namespace_privilege,
            app_privilege,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryUserPageParam {
    pub like_username: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub is_rev: bool,
}
