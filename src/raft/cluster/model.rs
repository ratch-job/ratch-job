use crate::app::model::{AppRouteRequest, AppRouteResponse};
use crate::raft::store::{ClientRequest, ClientResponse};
use actix::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub enum RouteAddr {
    Local,
    Remote(u64, Arc<String>),
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RouterRequest {
    JoinNode {
        node_id: u64,
        node_addr: Arc<String>,
    },
    AppRouteRequest(AppRouteRequest),
    RaftRequest(ClientRequest),
}

impl From<ClientRequest> for RouterRequest {
    fn from(req: ClientRequest) -> Self {
        RouterRequest::RaftRequest(req)
    }
}

impl From<AppRouteRequest> for RouterRequest {
    fn from(req: AppRouteRequest) -> Self {
        RouterRequest::AppRouteRequest(req)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RouterResponse {
    None,
    AppRouteResponse(AppRouteResponse),
    RaftResponse(ClientResponse),
}

impl From<ClientResponse> for RouterResponse {
    fn from(resp: ClientResponse) -> Self {
        RouterResponse::RaftResponse(resp)
    }
}

impl TryFrom<RouterResponse> for ClientResponse {
    type Error = anyhow::Error;

    fn try_from(value: RouterResponse) -> Result<Self, Self::Error> {
        match value {
            RouterResponse::RaftResponse(resp) => Ok(resp),
            _ => Err(anyhow::anyhow!("Invalid RaftResponse")),
        }
    }
}

impl From<AppRouteResponse> for RouterResponse {
    fn from(resp: AppRouteResponse) -> Self {
        RouterResponse::AppRouteResponse(resp)
    }
}

impl TryFrom<RouterResponse> for AppRouteResponse {
    type Error = anyhow::Error;
    fn try_from(value: RouterResponse) -> Result<Self, Self::Error> {
        match value {
            RouterResponse::AppRouteResponse(resp) => Ok(resp),
            _ => Err(anyhow::anyhow!("Invalid AppRouteResponse")),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VoteInfo {
    pub voted_for: u64,
    pub term: u64,
}

impl VoteInfo {
    pub fn new(voted_for: u64, term: u64) -> Self {
        VoteInfo { voted_for, term }
    }

    pub fn is_empty(&self) -> bool {
        self.voted_for == 0 && self.term == 0
    }
}

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<VoteChangeResponse>")]
pub enum VoteChangeRequest {
    VoteChange {
        vote_info: VoteInfo,
        local_is_master: bool,
    },
}

pub enum VoteChangeResponse {
    None,
}
