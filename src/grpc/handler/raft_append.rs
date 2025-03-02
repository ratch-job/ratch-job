use std::sync::Arc;

use crate::common::share_data::ShareData;
use crate::grpc::ratch_server_proto::Payload;
use crate::grpc::{HandlerResult, PayloadHandler, PayloadUtils, RequestMeta};
use crate::raft::store::ClientRequest;
use async_trait::async_trait;

pub struct RaftAppendRequestHandler {
    app_data: Arc<ShareData>,
}

impl RaftAppendRequestHandler {
    pub fn new(app_data: Arc<ShareData>) -> Self {
        Self { app_data }
    }
}

#[async_trait]
impl PayloadHandler for RaftAppendRequestHandler {
    async fn handle(
        &self,
        request_payload: Payload,
        _request_meta: RequestMeta,
    ) -> anyhow::Result<HandlerResult> {
        let body_vec = request_payload.body.unwrap_or_default().value;
        let request: async_raft_ext::raft::AppendEntriesRequest<ClientRequest> =
            serde_json::from_slice(&body_vec)?;
        let res = self.app_data.raft.append_entries(request).await?;
        let value = serde_json::to_vec(&res)?;
        let payload = PayloadUtils::build_payload("RaftAppendResponse", value);
        Ok(HandlerResult::success(payload))
    }
}
