use crate::raft::network::core::RaftRouter;
use crate::raft::store::core::Store;
use crate::raft::store::{ClientRequest, ClientResponse};
use async_raft_ext::raft::ClientWriteRequest;
use async_raft_ext::{Raft, RaftStorage};

pub mod cluster;
pub mod network;
pub mod store;

pub type RatchRaft = Raft<ClientRequest, ClientResponse, RaftRouter, Store>;

pub async fn join_node(raft: &RatchRaft, raft_store: &Store, node_id: u64) -> anyhow::Result<()> {
    let membership = raft_store.get_membership_config().await?;
    if !membership.contains(&node_id) {
        let mut all_node = membership.all_nodes();
        if all_node.contains(&node_id) {
            return Ok(());
        }
        all_node.insert(node_id);
        let members = all_node.clone().into_iter().collect();
        log::info!("join_node membership,{:?}", &all_node);
        raft.change_membership(all_node).await.ok();
        raft.client_write(ClientWriteRequest::new(ClientRequest::Members(members)))
            .await
            .unwrap();
    }
    Ok(())
}
