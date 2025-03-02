use crate::common::share_data::ShareData;
use crate::grpc::handler::raft_append::RaftAppendRequestHandler;
use crate::grpc::handler::raft_route::RaftRouteRequestHandler;
use crate::grpc::handler::raft_snapshot::RaftSnapshotRequestHandler;
use crate::grpc::handler::raft_vote::RaftVoteRequestHandler;
use crate::grpc::payload_utils::PayloadUtils;
use crate::grpc::ratch_server_proto::Payload;
use crate::grpc::{HandlerResult, PayloadHandler, RequestMeta};
use async_trait::async_trait;
use std::sync::Arc;

pub mod raft_append;
pub mod raft_route;
pub mod raft_snapshot;
pub mod raft_vote;

pub(crate) const CLUSTER_TOKEN: &str = "ClusterToken";
pub(crate) const RAFT_APPEND_REQUEST: &str = "RaftAppendRequest";
pub(crate) const RAFT_SNAPSHOT_REQUEST: &str = "RaftSnapshotRequest";
pub(crate) const RAFT_VOTE_REQUEST: &str = "RaftVoteRequest";
pub(crate) const RAFT_ROUTE_REQUEST: &str = "RaftRouteRequest";

pub struct InvokerHandler {
    app: Arc<ShareData>,
    handlers: Vec<(String, Box<dyn PayloadHandler + Send + Sync + 'static>)>,
}

impl InvokerHandler {
    pub fn new(app: Arc<ShareData>) -> Self {
        Self {
            app,
            handlers: vec![],
        }
    }

    pub fn add_handler(
        &mut self,
        url: &str,
        handler: Box<dyn PayloadHandler + Send + Sync + 'static>,
    ) {
        self.handlers.push((url.to_owned(), handler));
    }

    pub fn match_handler<'a>(
        &'a self,
        url: &str,
    ) -> Option<&'a (dyn PayloadHandler + Send + Sync + 'static)> {
        for (t, h) in &self.handlers {
            if t == url {
                return Some(h.as_ref());
            }
        }
        None
    }

    pub fn add_raft_handler(&mut self, app_data: &Arc<ShareData>) {
        self.add_handler(
            RAFT_APPEND_REQUEST,
            Box::new(RaftAppendRequestHandler::new(app_data.clone())),
        );
        self.add_handler(
            RAFT_SNAPSHOT_REQUEST,
            Box::new(RaftSnapshotRequestHandler::new(app_data.clone())),
        );
        self.add_handler(
            RAFT_VOTE_REQUEST,
            Box::new(RaftVoteRequestHandler::new(app_data.clone())),
        );
        self.add_handler(
            RAFT_ROUTE_REQUEST,
            Box::new(RaftRouteRequestHandler::new(app_data.clone())),
        );
    }
}

#[async_trait]
impl PayloadHandler for InvokerHandler {
    async fn handle(
        &self,
        request_payload: Payload,
        request_meta: RequestMeta,
    ) -> anyhow::Result<HandlerResult> {
        let url = request_payload.r#type.as_str();
        if let Some(handler) = self.match_handler(request_payload.r#type.as_str()) {
            return handler.handle(request_payload, request_meta).await;
        }
        log::warn!("InvokerHandler not fund handler,type:{}", url);
        Ok(HandlerResult::error(
            302u16,
            format!("{} RequestHandler Not Found", url),
        ))
    }
}
