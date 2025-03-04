use crate::common::datetime_utils::now_millis;
use crate::raft::network::factory::RaftClusterRequestSender;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    Valid,
    Invalid,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Valid
    }
}

#[derive(Default, Debug, Clone)]
pub struct ClusterNode {
    pub id: u64,
    pub index: u64,
    pub is_local: bool,
    pub addr: Arc<String>,
    pub status: NodeStatus,
}

#[derive(Default, Debug, Clone)]
pub struct ClusterInnerNode {
    pub id: u64,
    pub index: u64,
    pub is_local: bool,
    pub addr: Arc<String>,
    pub status: NodeStatus,
    pub last_active_time: u64,
}

impl ClusterInnerNode {
    pub(crate) fn is_valid(&self) -> bool {
        self.is_local || self.status == NodeStatus::Valid
    }
}

impl From<ClusterInnerNode> for ClusterNode {
    fn from(value: ClusterInnerNode) -> Self {
        Self {
            id: value.id,
            index: value.index,
            is_local: value.is_local,
            addr: value.addr,
            status: value.status,
        }
    }
}

#[bean(inject)]
pub struct ClusterNodeManager {
    local_id: u64,
    all_nodes: BTreeMap<u64, ClusterInnerNode>,
    cluster_sender: Option<Arc<RaftClusterRequestSender>>,
    first_init: bool,
}

impl ClusterNodeManager {
    pub fn new(local_id: u64) -> Self {
        Self {
            local_id,
            all_nodes: BTreeMap::new(),
            cluster_sender: None,
            first_init: false,
        }
    }

    fn update_nodes(&mut self, nodes: Vec<(u64, Arc<String>)>, ctx: &mut Context<Self>) {
        if self.cluster_sender.is_none() {
            log::warn!("InnerNodeManage cluster_sender is none");
            return;
        }
        let new_sets: HashSet<u64> = nodes.iter().map(|e| e.0.to_owned()).collect();
        let mut dels = vec![];
        for key in self.all_nodes.keys() {
            if !new_sets.contains(key) {
                dels.push(*key);
            }
        }
        for key in dels {
            self.all_nodes.remove(&key);
        }
        let now = now_millis();
        for (key, addr) in nodes {
            if let Some(node) = self.all_nodes.get_mut(&key) {
                node.addr = addr;
            } else {
                let is_local = self.local_id == key;
                let node = ClusterInnerNode {
                    id: key,
                    index: 0,
                    is_local,
                    addr,
                    status: NodeStatus::Valid,
                    last_active_time: now,
                };
                self.all_nodes.insert(key, node);
            }
        }
        let local_node = self.get_this_node();
        self.all_nodes.entry(self.local_id).or_insert(local_node);
        self.update_nodes_index();
        //第一次需要触发从其它实例加载snapshot
        if !self.first_init {
            self.first_init = true;
            ctx.run_later(Duration::from_millis(1000), |act, _ctx| {
                act.load_snapshot_from_node();
            });
        }
    }

    fn update_nodes_index(&mut self) {
        for (i, value) in self.all_nodes.values_mut().enumerate() {
            value.index = i as u64;
        }
    }

    fn load_snapshot_from_node(&self) {
        //todo 触发从主节点加载已注册应用实例
    }

    fn get_this_node(&self) -> ClusterInnerNode {
        if let Some(node) = self.all_nodes.get(&self.local_id) {
            node.to_owned()
        } else {
            ClusterInnerNode {
                id: self.local_id,
                is_local: true,
                ..Default::default()
            }
        }
    }

    fn get_all_nodes(&self) -> Vec<ClusterNode> {
        if self.all_nodes.is_empty() {
            vec![self.get_this_node().into()]
        } else {
            self.all_nodes.values().cloned().map(|e| e.into()).collect()
        }
    }
}

impl Actor for ClusterNodeManager {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("ClusterNodeManager started!");
    }
}

impl Inject for ClusterNodeManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.cluster_sender = factory_data.get_bean();
    }
}

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<NodeManageResponse>")]
pub enum NodeManageRequest {
    UpdateNodes(Vec<(u64, Arc<String>)>),
    GetThisNode,
    GetAllNodes,
    GetNode(u64),
    //SendToOtherNodes
}

pub enum NodeManageResponse {
    None,
    ThisNode(ClusterNode),
    Node(Option<ClusterNode>),
    AllNodes(Vec<ClusterNode>),
}

impl Handler<NodeManageRequest> for ClusterNodeManager {
    type Result = anyhow::Result<NodeManageResponse>;

    fn handle(&mut self, msg: NodeManageRequest, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            NodeManageRequest::UpdateNodes(nodes) => {
                log::info!("InnerNodeManage UpdateNodes,size:{}", nodes.len());
                self.update_nodes(nodes, ctx);
                Ok(NodeManageResponse::None)
            }
            NodeManageRequest::GetThisNode => {
                Ok(NodeManageResponse::ThisNode(self.get_this_node().into()))
            }
            NodeManageRequest::GetAllNodes => {
                Ok(NodeManageResponse::AllNodes(self.get_all_nodes()))
            }
            NodeManageRequest::GetNode(node_id) => {
                let node = self.all_nodes.get(&node_id).map(|e| e.to_owned().into());
                Ok(NodeManageResponse::Node(node))
            }
        }
    }
}
