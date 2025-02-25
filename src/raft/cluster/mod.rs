pub mod model;
pub mod route;

use crate::common::share_data::ShareData;
use crate::raft::cluster::model::{RouterRequest, RouterResponse};
use crate::raft::join_node;
use crate::raft::store::ClientRequest;
use async_raft_ext::raft::ClientWriteRequest;
use std::sync::Arc;

pub async fn handle_route(
    app: &Arc<ShareData>,
    req: RouterRequest,
) -> anyhow::Result<RouterResponse> {
    match req {
        RouterRequest::JoinNode {
            node_id,
            node_addr: addr,
        } => {
            app.raft
                .client_write(ClientWriteRequest::new(ClientRequest::NodeAddr {
                    id: node_id,
                    addr,
                }))
                .await?;
            app.raft.add_non_voter(node_id).await?;
            join_node(app.raft.as_ref(), app.raft_store.as_ref(), node_id).await?;
            Ok(RouterResponse::None)
        }
        RouterRequest::RaftRequest(req) => {
            let r = app.raft.client_write(ClientWriteRequest::new(req)).await?;
            Ok(RouterResponse::RaftResponse(r.data))
        }
    }
}
