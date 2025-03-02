use std::{fmt::Debug, sync::Arc};

use super::model::{RouteAddr, RouterRequest, RouterResponse};
use crate::grpc::handler::RAFT_ROUTE_REQUEST;
use crate::raft::store::core::Store;
use crate::raft::store::{ClientRequest, ClientResponse};
use crate::raft::RatchRaft;
use crate::{grpc::PayloadUtils, raft::network::factory::RaftClusterRequestSender};
use actix::prelude::*;
use async_raft_ext::raft::{ClientWriteRequest, ClientWriteResponse};

#[derive(Clone)]
pub struct RaftAddrRouter {
    raft_store: Arc<Store>,
    raft: Arc<RatchRaft>,
    local_node_id: u64,
}

impl Debug for RaftAddrRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddrRouter").finish()
    }
}

impl RaftAddrRouter {
    pub fn new(raft: Arc<RatchRaft>, raft_store: Arc<Store>, local_node_id: u64) -> Self {
        Self {
            raft,
            raft_store,
            local_node_id,
        }
    }

    pub async fn get_route_addr(&self) -> anyhow::Result<RouteAddr> {
        //let state = self.raft_store.get_initial_state().await?;
        let leader = self.raft.current_leader().await;
        match leader {
            Some(node_id) => {
                if node_id == self.local_node_id {
                    Ok(RouteAddr::Local)
                } else {
                    let addr = self.raft_store.get_target_addr(node_id).await?;
                    Ok(RouteAddr::Remote(node_id, addr))
                }
            }
            None => Ok(RouteAddr::Unknown),
        }
    }
}

///
/// raft 请求路由
/// 考虑都使用这个对象统一处理；
#[derive(Clone)]
pub struct RaftRequestRoute {
    raft_addr_route: Arc<RaftAddrRouter>,
    cluster_sender: Arc<RaftClusterRequestSender>,
    raft: Arc<RatchRaft>,
}

impl RaftRequestRoute {
    pub fn new(
        raft_addr_route: Arc<RaftAddrRouter>,
        cluster_sender: Arc<RaftClusterRequestSender>,
        raft: Arc<RatchRaft>,
    ) -> Self {
        Self {
            raft_addr_route,
            cluster_sender,
            raft,
        }
    }

    fn unknown_err(&self) -> anyhow::Error {
        anyhow::anyhow!("unknown the raft leader addr!")
    }

    pub async fn request(&self, req: ClientRequest) -> anyhow::Result<ClientResponse> {
        match self.raft_addr_route.get_route_addr().await? {
            RouteAddr::Local => {
                let resp = self.raft.client_write(ClientWriteRequest::new(req)).await?;
                Ok(resp.data)
            }
            RouteAddr::Remote(_, addr) => {
                let req: RouterRequest = req.into();
                let request = serde_json::to_vec(&req).unwrap_or_default();
                let payload = PayloadUtils::build_payload(RAFT_ROUTE_REQUEST, request);
                let resp_payload = self.cluster_sender.send_request(addr, payload).await?;
                let body_vec = resp_payload.body.unwrap_or_default().value;
                let router_resp: RouterResponse = serde_json::from_slice(&body_vec)?;
                let resp: ClientResponse = router_resp.try_into()?;
                Ok(resp)
            }
            RouteAddr::Unknown => Err(self.unknown_err()),
        }
    }
}
