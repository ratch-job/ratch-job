use crate::raft::store::{ClientRequest, ClientResponse};
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
    RaftRequest(ClientRequest),
}

impl From<ClientRequest> for RouterRequest {
    fn from(req: ClientRequest) -> Self {
        RouterRequest::RaftRequest(req)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RouterResponse {
    None,
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
