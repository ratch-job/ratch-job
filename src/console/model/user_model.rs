use crate::common::datetime_utils::now_millis_i64;
use crate::common::model::privilege::{PrivilegeGroup, PrivilegeGroupOptionParam};
use crate::user::model::{QueryUserPageParam, UserDto, UserInfo};
use crate::user::permission::UserRoleHelper;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserPermissions {
    pub resources: Vec<&'static str>,
    pub from: &'static str,
    pub version: &'static str,
    pub username: Option<Arc<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordParam {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserSimpleVO {
    pub username: Option<Arc<String>>,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserVO {
    pub username: Arc<String>,
    pub nickname: String,
    pub gmt_create: i64,
    pub gmt_modified: i64,
    pub enable: bool,
    pub roles: Vec<Arc<String>>,
    pub extend_info: HashMap<String, String>,
    pub namespace_privilege: PrivilegeGroup<Arc<String>>,
    pub app_privilege: PrivilegeGroup<Arc<String>>,
}

impl From<UserInfo> for UserVO {
    fn from(value: UserInfo) -> Self {
        Self {
            username: value.username,
            nickname: value.nickname,
            gmt_create: value.gmt_create,
            gmt_modified: value.gmt_modified,
            enable: value.enable,
            roles: value.roles,
            extend_info: value.extend_info,
            namespace_privilege: value.namespace_privilege,
            app_privilege: value.app_privilege,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserInfoParam {
    pub username: Arc<String>,
    pub nickname: Option<String>,
    pub password: Option<String>,
    pub enable: Option<bool>,
    pub roles: Option<String>,
    pub namespace_privilege_param: Option<PrivilegeGroupOptionParam<Arc<String>>>,
    pub app_privilege_param: Option<PrivilegeGroupOptionParam<Arc<String>>>,
}

impl UpdateUserInfoParam {
    pub fn get_role_vec(&self) -> Option<Vec<Arc<String>>> {
        if let Some(roles) = self.roles.as_ref() {
            if roles.is_empty() {
                return None;
            }
            Some(roles.split(',').map(UserRoleHelper::get_role).collect())
        } else {
            None
        }
    }
}

impl From<UpdateUserInfoParam> for UserDto {
    fn from(value: UpdateUserInfoParam) -> Self {
        let roles = value.get_role_vec();
        let now = now_millis_i64();
        Self {
            username: value.username,
            nickname: value.nickname,
            password: value.password,
            password_hash: None,
            gmt_create: Some(now),
            gmt_modified: Some(now),
            enable: value.enable,
            roles,
            app_privilege: value.app_privilege_param,
            namespace_privilege: value.namespace_privilege_param,
            extend_info: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPageParams {
    pub like_username: Option<String>,
    pub is_rev: Option<bool>,
    pub page_no: Option<usize>,
    pub page_size: Option<usize>,
}

impl From<UserPageParams> for QueryUserPageParam {
    fn from(param: UserPageParams) -> Self {
        let limit = param.page_size.unwrap_or(0xffff_ffff);
        let mut page_no = param.page_no.unwrap_or(1);
        if page_no == 0 {
            page_no = 1;
        }
        let offset = (page_no - 1) * limit;
        Self {
            like_username: param.like_username,
            is_rev: param.is_rev.unwrap_or(false),
            limit: Some(limit as i64),
            offset: Some(offset as i64),
        }
    }
}
