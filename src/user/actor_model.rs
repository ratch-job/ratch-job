use crate::user::model::{QueryUserPageParam, UserDto, UserInfo};
use actix::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<UserManagerRaftResult>")]
pub enum UserManagerReq {
    CheckUser { name: Arc<String>, password: String },
    Query { name: Arc<String> },
    QueryPageList(QueryUserPageParam),
}

#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "anyhow::Result<UserManagerRaftResult>")]
pub enum UserManagerRaftReq {
    AddUser(UserDto),
    UpdateUser(UserDto),
    CheckUser { name: Arc<String>, password: String },
    Remove(Arc<String>),
    Query { name: Arc<String> },
    QueryPageList(QueryUserPageParam),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserManagerRaftResult {
    None,
    QueryUser(Option<UserInfo>),
    CheckUser(bool, UserInfo),
    UserPage(usize, Vec<UserInfo>),
}
