use crate::grpc::handler::{RAFT_APPEND_REQUEST, RAFT_SNAPSHOT_REQUEST, RAFT_VOTE_REQUEST};
use async_raft_ext::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use async_raft_ext::{NodeId, RaftNetwork};
use async_trait::async_trait;
use std::sync::Arc;

use super::factory::RaftClusterRequestSender;
use crate::grpc::ratch_server_proto::Payload;
use crate::grpc::PayloadUtils;
use crate::raft::store::core::Store;
use crate::raft::store::ClientRequest;

pub struct RaftRouter {
    store: Arc<Store>, //get target addr
    cluster_sender: Arc<RaftClusterRequestSender>,
}

impl RaftRouter {
    pub fn new(store: Arc<Store>, cluster_sender: Arc<RaftClusterRequestSender>) -> Self {
        Self {
            store,
            cluster_sender,
        }
    }

    async fn send_request(&self, target: u64, payload: Payload) -> anyhow::Result<Payload> {
        let addr = self.store.get_target_addr(target).await?;
        self.cluster_sender.send_request(addr, payload).await
    }
}

#[async_trait]
impl RaftNetwork<ClientRequest> for RaftRouter {
    async fn append_entries(
        &self,
        target: NodeId,
        req: AppendEntriesRequest<ClientRequest>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let request = serde_json::to_vec(&req).unwrap_or_default();
        let payload = PayloadUtils::build_payload(RAFT_APPEND_REQUEST, request);
        let resp_payload = self.send_request(target, payload).await?;
        let body_vec = resp_payload.body.unwrap_or_default().value;
        let res: AppendEntriesResponse = serde_json::from_slice(&body_vec)?;
        Ok(res)
    }

    async fn install_snapshot(
        &self,
        target: NodeId,
        req: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        let request = serde_json::to_vec(&req).unwrap_or_default();
        let payload = PayloadUtils::build_payload(RAFT_SNAPSHOT_REQUEST, request);
        let resp_payload = self.send_request(target, payload).await?;
        let body_vec = resp_payload.body.unwrap_or_default().value;
        let res: InstallSnapshotResponse = serde_json::from_slice(&body_vec)?;
        Ok(res)
    }

    async fn vote(&self, target: NodeId, req: VoteRequest) -> anyhow::Result<VoteResponse> {
        let request = serde_json::to_vec(&req).unwrap_or_default();
        let payload = PayloadUtils::build_payload(RAFT_VOTE_REQUEST, request);
        let resp_payload = self.send_request(target, payload).await?;
        let body_vec = resp_payload.body.unwrap_or_default().value;
        let res: VoteResponse = serde_json::from_slice(&body_vec)?;
        Ok(res)
    }
}
