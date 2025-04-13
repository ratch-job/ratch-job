use crate::common::constant::USER_TABLE_NAME;
use crate::common::pb::data_object::UserInfoDo;
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::{RaftApplyDataRequest, RaftApplyDataResponse};
use crate::raft::store::raftsnapshot::{SnapshotWriterActor, SnapshotWriterRequest};
use crate::user::actor_model::{UserManagerRaftReq, UserManagerRaftResult, UserManagerReq};
use crate::user::build_password_hash;
use crate::user::model::{QueryUserPageParam, UserDto, UserInfo};
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use quick_protobuf::{BytesReader, Writer};
use std::collections::BTreeMap;
use std::sync::Arc;

#[bean(inject)]
pub struct UserManager {
    data: BTreeMap<Arc<String>, UserInfo>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            data: BTreeMap::new(),
        }
    }

    fn add_user(&mut self, user_dto: UserDto) -> bool {
        let user: UserInfo = user_dto.into();
        self.data.insert(user.username.clone(), user);
        true
    }

    fn update_user(&mut self, user_dto: UserDto) -> bool {
        if let Some(user) = self.data.get_mut(&user_dto.username) {
            user.update(user_dto);
            true
        } else {
            self.add_user(user_dto);
            false
        }
    }

    fn check_user(&self, name: Arc<String>, password: String) -> anyhow::Result<(bool, UserInfo)> {
        if let Some(user) = self.data.get(&name) {
            let password_hash = build_password_hash(&password)?;
            if user.password_hash == password_hash {
                Ok((true, user.clone()))
            } else {
                Ok((false, user.clone()))
            }
        } else {
            Err(anyhow::anyhow!("user not found"))
        }
    }

    fn remove_user(&mut self, username: Arc<String>) -> bool {
        self.data.remove(&username).is_some()
    }

    fn query_page(&self, param: QueryUserPageParam) -> (usize, Vec<UserInfo>) {
        let mut users: Vec<&UserInfo> = self.data.values().collect();

        // 根据 like_username 过滤用户
        if let Some(like_username) = param.like_username {
            users.retain(|user| user.username.contains(&like_username));
        }

        // 根据 is_rev 决定是否反转排序
        if param.is_rev {
            users.reverse();
        }

        // 计算 offset 和 limit
        let offset = param.offset.unwrap_or(0) as usize;
        let limit = param.limit.unwrap_or(users.len() as i64) as usize;

        // 分页
        let total_count = users.len();
        let paged_users = users
            .into_iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        (total_count, paged_users)
    }

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        for (key, v) in self.data.iter() {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf);
                let value_do = v.to_do();
                writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: USER_TABLE_NAME.clone(),
                key: key.to_string().into_bytes(),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        Ok(())
    }

    fn load_snapshot_record(&mut self, record: SnapshotRecordDto) -> anyhow::Result<()> {
        let mut reader = BytesReader::from_bytes(&record.value);
        let value_do: UserInfoDo = reader.read_message(&record.value)?;
        let user_info: UserInfo = value_do.into();
        self.data.insert(user_info.username.clone(), user_info);
        Ok(())
    }

    fn load_completed(&mut self, _ctx: &mut Context<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Actor for UserManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("UserManager started");
    }
}

impl Inject for UserManager {
    type Context = Context<Self>;

    fn inject(&mut self, factory_data: FactoryData, factory: BeanFactory, ctx: &mut Self::Context) {
    }
}

impl Handler<UserManagerReq> for UserManager {
    type Result = anyhow::Result<UserManagerRaftResult>;

    fn handle(&mut self, msg: UserManagerReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            UserManagerReq::CheckUser { name, password } => {
                if let Ok((is_ok, user)) = self.check_user(name, password) {
                    Ok(UserManagerRaftResult::CheckUser(is_ok, user))
                } else {
                    Ok(UserManagerRaftResult::None)
                }
            }
            UserManagerReq::Query { name } => {
                let v = self.data.get(&name);
                Ok(UserManagerRaftResult::QueryUser(v.map(|v| v.clone())))
            }
            UserManagerReq::QueryPageList(param) => {
                let (total_count, paged_users) = self.query_page(param);
                Ok(UserManagerRaftResult::UserPage(total_count, paged_users))
            }
        }
    }
}

impl Handler<UserManagerRaftReq> for UserManager {
    type Result = anyhow::Result<UserManagerRaftResult>;

    fn handle(&mut self, msg: UserManagerRaftReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            UserManagerRaftReq::AddUser(user_dto) => {
                self.add_user(user_dto);
                Ok(UserManagerRaftResult::None)
            }
            UserManagerRaftReq::UpdateUser(user_dto) => {
                self.update_user(user_dto);
                Ok(UserManagerRaftResult::None)
            }
            UserManagerRaftReq::CheckUser { name, password } => {
                if let Ok((is_ok, user)) = self.check_user(name, password) {
                    Ok(UserManagerRaftResult::CheckUser(is_ok, user))
                } else {
                    Ok(UserManagerRaftResult::None)
                }
            }
            UserManagerRaftReq::Remove(username) => {
                self.remove_user(username);
                Ok(UserManagerRaftResult::None)
            }
            UserManagerRaftReq::Query { name } => {
                let v = self.data.get(&name);
                Ok(UserManagerRaftResult::QueryUser(v.map(|v| v.clone())))
            }
            UserManagerRaftReq::QueryPageList(param) => {
                let (total_count, paged_users) = self.query_page(param);
                Ok(UserManagerRaftResult::UserPage(total_count, paged_users))
            }
        }
    }
}

impl Handler<RaftApplyDataRequest> for UserManager {
    type Result = anyhow::Result<RaftApplyDataResponse>;

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
