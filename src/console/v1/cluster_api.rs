use std::sync::Arc;

use crate::common::model::ApiResult;
use crate::common::share_data::ShareData;
use crate::console::model::cluster_model::ClusterNodeInfo;
use crate::raft::cluster::node_manager::{ClusterNode, NodeManageRequest, NodeManageResponse};
use actix_web::{http::header, web, HttpResponse, Responder};

async fn get_all_valid_nodes(app: &Arc<ShareData>) -> anyhow::Result<Vec<ClusterNode>> {
    if let NodeManageResponse::AllNodes(v) = app
        .cluster_node_manager
        .send(NodeManageRequest::GetAllNodes)
        .await??
    {
        Ok(v)
    } else {
        Err(anyhow::anyhow!("get_all_valid_nodes error"))
    }
}

pub async fn query_cluster_info(app: web::Data<Arc<ShareData>>) -> impl Responder {
    let nodes = get_all_valid_nodes(&app).await.unwrap();
    let leader_node = app.raft.current_leader().await;
    let mut list = vec![];
    for node in nodes {
        let mut node_info: ClusterNodeInfo = node.into();
        if let Some(leader_node) = &leader_node {
            if node_info.node_id == *leader_node {
                node_info.raft_leader = true;
            }
        }
        if app.app_config.raft_node_id == node_info.node_id {
            node_info.current_node = true;
        }
        list.push(node_info);
    }
    HttpResponse::Ok().json(ApiResult::success(Some(list)))
}
