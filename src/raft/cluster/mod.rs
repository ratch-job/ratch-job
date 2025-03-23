pub mod model;
pub mod node_manager;
pub mod route;

use crate::app::model::{AppManagerReq, AppManagerResult};
use crate::common::share_data::ShareData;
use crate::grpc::handler::RAFT_ROUTE_REQUEST;
use crate::grpc::PayloadUtils;
use crate::metrics::model::{MetricsRequest, MetricsResponse};
use crate::raft::cluster::model::{RouterRequest, RouterResponse};
use crate::raft::join_node;
use crate::raft::network::factory::RaftClusterRequestSender;
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
        RouterRequest::AppRouteRequest(req) => {
            if let AppManagerResult::AppRouteResponse(resp) = app
                .app_manager
                .send(AppManagerReq::AppRouteRequest(req))
                .await??
            {
                Ok(RouterResponse::AppRouteResponse(resp))
            } else {
                Err(anyhow::anyhow!("AppManagerReq::AppRouteRequest error"))
            }
        }
        RouterRequest::MetricsTimelineQuery(param) => {
            if let MetricsResponse::TimelineResponse(resp) = app
                .metrics_manager
                .send(MetricsRequest::TimelineQuery(param))
                .await??
            {
                Ok(RouterResponse::MetricsTimeLineResponse(resp))
            } else {
                Err(anyhow::anyhow!("MetricsResponse::TimelineResponse error"))
            }
        }
    }
}

pub async fn router_request(
    req: RouterRequest,
    addr: Arc<String>,
    cluster_sender: &Arc<RaftClusterRequestSender>,
) -> anyhow::Result<RouterResponse> {
    let request = serde_json::to_vec(&req).unwrap_or_default();
    let payload = PayloadUtils::build_payload(RAFT_ROUTE_REQUEST, request);
    let resp_payload = cluster_sender.send_request(addr, payload).await?;
    let body_vec = resp_payload.body.unwrap_or_default().value;
    let router_resp: RouterResponse = serde_json::from_slice(&body_vec)?;
    Ok(router_resp)
}
