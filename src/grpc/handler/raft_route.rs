use std::sync::Arc;

use crate::common::share_data::ShareData;
use crate::grpc::ratch_server_proto::Payload;
use crate::grpc::{HandlerResult, PayloadHandler, PayloadUtils, RequestMeta};
use crate::raft::cluster::handle_route;
use crate::raft::cluster::model::RouterRequest;
use async_trait::async_trait;

pub struct RaftRouteRequestHandler {
    app_data: Arc<ShareData>,
}

impl RaftRouteRequestHandler {
    pub fn new(app_data: Arc<ShareData>) -> Self {
        Self { app_data }
    }
}

#[async_trait]
impl PayloadHandler for RaftRouteRequestHandler {
    async fn handle(
        &self,
        request_payload: Payload,
        _request_meta: RequestMeta,
    ) -> anyhow::Result<HandlerResult> {
        let body_vec = request_payload.body.unwrap_or_default().value;
        let request: RouterRequest = serde_json::from_slice(&body_vec)?;
        let res = handle_route(&self.app_data, request).await?;
        let value = serde_json::to_vec(&res)?;
        let payload = PayloadUtils::build_payload("RaftRouteResponse", value);
        Ok(HandlerResult::success(payload))
    }
}
