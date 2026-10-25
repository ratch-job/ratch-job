use crate::common::datetime_utils::now_millis_i64;
use crate::common::string_utils::StringUtils;
use crate::ldap::model::actor_model::{LdapMsgActorReq, LdapMsgReq, LdapMsgResult};
use crate::ldap::model::{LdapConfig, LdapUserMeta};
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::ClientRequest;
use crate::user::actor_model::{UserManagerRaftReq, UserManagerRaftResult, UserManagerReq};
use crate::user::model::{UserDto, UserInfo};
use crate::user::permission;
use crate::user::UserManager;
use actix::prelude::*;
use ldap3::{Ldap, SearchEntry};
use std::sync::Arc;

pub struct LdapMsgActor {
    ldap: Option<Ldap>,
    ldap_version: u64,
    /// ldap备份；ldap每次使用时使用值传递，减少clone次数；ldap_backup用于并发时ldap为None后可以使用；
    ldap_backup: Option<Ldap>,
    ldap_config: Arc<LdapConfig>,
    user_manager_addr: Option<Addr<UserManager>>,
    raft_request_route: Option<Arc<RaftRequestRoute>>,
}

async fn get_or_add_user(
    user_manager_addr: &Addr<UserManager>,
    raft_request_route: Option<&Arc<RaftRequestRoute>>,
    user_name: &str,
    role: Arc<String>,
) -> Option<UserInfo> {
    let name = Arc::new(user_name.to_owned());
    if let Ok(Ok(UserManagerRaftResult::QueryUser(Some(user_info)))) = user_manager_addr
        .send(UserManagerReq::Query { name: name.clone() })
        .await
    {
        return Some(user_info);
    }
    // 用户不存在时通过raft添加用户，保证数据持久化与集群复制
    let now = now_millis_i64();
    let user = UserDto {
        username: name,
        nickname: Some(user_name.to_owned()),
        enable: Some(true),
        gmt_create: Some(now),
        gmt_modified: Some(now),
        roles: Some(vec![role]),
        ..Default::default()
    };
    let mut added = false;
    if let Some(raft_request_route) = raft_request_route {
        let req = UserManagerRaftReq::AddUser(user);
        match raft_request_route
            .request(ClientRequest::UserReq { req })
            .await
        {
            Ok(_) => {
                added = true;
            }
            Err(e) => {
                log::error!("ldap add user by raft error: {}", e);
            }
        }
    }
    if !added {
        return None;
    }
    // 添加成功后重新查询获取用户信息；查询不到时使用默认权限
    if let Ok(Ok(UserManagerRaftResult::QueryUser(Some(user_info)))) = user_manager_addr
        .send(UserManagerReq::Query {
            name: Arc::new(user_name.to_owned()),
        })
        .await
    {
        return Some(user_info);
    }
    None
}

impl LdapMsgActor {
    pub(crate) fn new(
        ldap: Ldap,
        ldap_config: Arc<LdapConfig>,
        user_manager_addr: Option<Addr<UserManager>>,
        raft_request_route: Option<Arc<RaftRequestRoute>>,
    ) -> Self {
        let ldap_backup = Some(ldap.clone());
        Self {
            ldap: Some(ldap),
            ldap_version: 0,
            ldap_backup,
            ldap_config,
            user_manager_addr,
            raft_request_route,
        }
    }

    fn set_ldap(&mut self, ldap: Ldap) {
        self.ldap_backup = Some(ldap.clone());
        self.ldap = Some(ldap);
        self.ldap_version += 1;
    }

    fn get_default_user_meta(user_name: String, ldap_config: Arc<LdapConfig>) -> LdapUserMeta {
        LdapUserMeta::new(
            user_name,
            vec![],
            ldap_config.ldap_user_default_role.clone(),
            Default::default(),
            Default::default(),
        )
    }

    async fn handle_req(
        ldap: &mut Ldap,
        ldap_config: Arc<LdapConfig>,
        msg: LdapMsgReq,
        user_manager_addr: Option<Addr<UserManager>>,
        raft_request_route: Option<Arc<RaftRequestRoute>>,
    ) -> anyhow::Result<LdapMsgResult> {
        match msg {
            LdapMsgReq::Bind(bind_req) => {
                let bind_dn = format!(
                    "uid={},{}",
                    bind_req.user_name, &ldap_config.ldap_user_base_dn
                );
                ldap.simple_bind(&bind_dn, &bind_req.password)
                    .await?
                    .success()?;
                let filter = ldap_config
                    .ldap_user_filter
                    .replace("%s", &bind_req.user_name);
                let (mut rs, _res) = match ldap
                    .search(
                        &ldap_config.ldap_user_base_dn,
                        ldap3::Scope::Subtree,
                        &filter,
                        vec!["cn", "memberOf"],
                    )
                    .await?
                    .success()
                {
                    Ok(res) => res,
                    Err(_e) => {
                        let meta = Self::get_default_user_meta(bind_req.user_name, ldap_config);
                        return Ok(LdapMsgResult::UserMeta(meta));
                    }
                };
                if !rs.is_empty() {
                    let entry = rs.remove(0);
                    let mut entry = SearchEntry::construct(entry);
                    let mut role = ldap_config.ldap_user_default_role.clone();
                    let mut namespace_privilege = None;
                    let mut app_privilege = None;
                    let groups = entry.attrs.remove("memberOf").unwrap_or_default();
                    let groups = groups
                        .iter()
                        .map(|s| StringUtils::extract_ldap_value_cn(s))
                        .filter(|s| s.is_some())
                        .map(|s| s.unwrap())
                        .collect::<Vec<_>>();
                    for group in groups.iter() {
                        if ldap_config.ldap_user_developer_groups.contains(group) {
                            role = permission::USER_ROLE_DEVELOPER.clone();
                            break;
                        }
                    }
                    for group in groups.iter() {
                        if ldap_config.ldap_user_admin_groups.contains(group) {
                            role = permission::USER_ROLE_MANAGER.clone();
                            break;
                        }
                    }
                    if let Some(user_manager_addr) = user_manager_addr {
                        //init user
                        if let Some(user_info) = get_or_add_user(
                            &user_manager_addr,
                            raft_request_route.as_ref(),
                            &bind_req.user_name,
                            role.clone(),
                        )
                        .await
                        {
                            namespace_privilege = Some(user_info.namespace_privilege);
                            app_privilege = Some(user_info.app_privilege);
                        }
                    }
                    let meta = LdapUserMeta::new(
                        bind_req.user_name,
                        groups,
                        role,
                        namespace_privilege.unwrap_or_default(),
                        app_privilege.unwrap_or_default(),
                    );
                    Ok(LdapMsgResult::UserMeta(meta))
                } else {
                    let meta = Self::get_default_user_meta(bind_req.user_name, ldap_config);
                    Ok(LdapMsgResult::UserMeta(meta))
                }
            }
        }
    }
}

impl Actor for LdapMsgActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("LdapMsgActor started");
    }
}

impl Handler<LdapMsgReq> for LdapMsgActor {
    type Result = ResponseActFuture<Self, anyhow::Result<LdapMsgResult>>;

    fn handle(&mut self, msg: LdapMsgReq, ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let mut ldap = self.ldap.take();
        if ldap.is_none() {
            ldap = self.ldap_backup.clone();
        }
        let version = self.ldap_version;
        let config = self.ldap_config.clone();
        let user_manager_addr = self.user_manager_addr.clone();
        let raft_request_route = self.raft_request_route.clone();
        async move {
            let r = if let Some(ldap) = ldap.as_mut() {
                Self::handle_req(ldap, config, msg, user_manager_addr, raft_request_route).await
            } else {
                Err(anyhow::anyhow!("ldap not init"))
            };
            if let Err(e) = r.as_ref() {
                log::error!("handle ldap msg error: {}", e);
            }
            (ldap, version, tx, r)
        }
        .into_actor(self)
        .map(|(ldap, version, tx, r), act, _ctx| {
            if act.ldap_version == version {
                act.ldap = ldap;
            }
            tx.send(r).ok();
        })
        .wait(ctx); //使用wait执行，保证handle_req内能依次执行，避免出现异步插队可能引发的问题。

        let fut = rx.into_actor(self).map(|r, _act, _ctx| match r {
            Ok(v) => v,
            Err(r) => Err(anyhow::anyhow!(r.to_string())),
        });
        Box::pin(fut)
    }
}

impl Handler<LdapMsgActorReq> for LdapMsgActor {
    type Result = anyhow::Result<()>;
    fn handle(&mut self, msg: LdapMsgActorReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            LdapMsgActorReq::SetLdap(ldap) => {
                self.set_ldap(ldap);
            }
        }
        Ok(())
    }
}
