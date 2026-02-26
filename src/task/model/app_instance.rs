use crate::app::model::AppKey;
use crate::common::hash_utils::get_hash_value;
use crate::job::model::enum_type::RouterStrategy;
use rand::prelude::SliceRandom;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

pub enum InstanceAddrSelectResult {
    Fixed(Arc<String>),
    Selected(Arc<String>),
    ALL(Arc<Vec<Arc<String>>>),
    Empty,
}

/// 一致性哈希环的虚拟节点数
const CONSISTENT_HASH_VIRTUAL_NODES: u32 = 160;

/// 一致性哈希环，使用虚拟节点实现均匀分布
#[derive(Clone, Debug)]
pub struct ConsistentHashRing {
    ring: BTreeMap<u64, Arc<String>>,
}

impl ConsistentHashRing {
    pub fn new() -> Self {
        ConsistentHashRing {
            ring: BTreeMap::new(),
        }
    }

    pub fn build(instance_keys: &[Arc<String>]) -> Self {
        let mut ring = BTreeMap::new();
        for key in instance_keys {
            for i in 0..CONSISTENT_HASH_VIRTUAL_NODES {
                let virtual_key = format!("{}#{}", key, i);
                let hash = get_hash_value(&virtual_key);
                ring.insert(hash, key.clone());
            }
        }
        ConsistentHashRing { ring }
    }

    /// 根据 job_id 的哈希值在环上顺时针查找最近的节点
    pub fn get_instance(&self, job_id: u64) -> Option<Arc<String>> {
        if self.ring.is_empty() {
            return None;
        }
        let hash = get_hash_value(&job_id);
        // 顺时针查找第一个大于等于 hash 的节点，找不到则绕回环首
        self.ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, v)| v.clone())
    }
}

#[derive(Clone, Debug)]
pub struct AppInstanceState {
    pub addr: Arc<String>,
    pub error_times: u16,
    pub first_error_time: u32,
}

impl AppInstanceState {
    pub fn new(addr: Arc<String>) -> Self {
        AppInstanceState {
            addr,
            error_times: 0,
            first_error_time: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppInstanceStateGroup {
    pub app_key: AppKey,
    pub instance_map: HashMap<Arc<String>, AppInstanceState>,
    pub instance_keys: Arc<Vec<Arc<String>>>,
    pub round_robin_index: usize,
    pub hash_ring: ConsistentHashRing,
}

impl AppInstanceStateGroup {
    pub fn new(app_key: AppKey) -> Self {
        AppInstanceStateGroup {
            app_key,
            instance_map: HashMap::new(),
            instance_keys: Arc::new(Vec::new()),
            round_robin_index: 0,
            hash_ring: ConsistentHashRing::new(),
        }
    }

    pub fn clean(&mut self) {
        self.round_robin_index = 0;
        self.instance_map = HashMap::new();
        self.instance_keys = Arc::new(Vec::new());
        self.hash_ring = ConsistentHashRing::new();
    }

    fn rebuild_hash_ring(&mut self) {
        self.hash_ring = ConsistentHashRing::build(&self.instance_keys);
    }

    pub fn set_instance_list(&mut self, instance_list: Vec<Arc<String>>) {
        for key in instance_list {
            self.add_instance(key);
        }
    }

    pub fn add_instance(&mut self, key: Arc<String>) {
        if self.instance_map.contains_key(&key) {
            return;
        }
        let instance = AppInstanceState::new(key.clone());
        self.instance_map.insert(key.clone(), instance);
        self.instance_keys = Arc::new(self.instance_map.keys().map(|k| k.clone()).collect());
        self.rebuild_hash_ring();
    }

    pub fn remove_instance(&mut self, key: Arc<String>) {
        if !self.instance_map.contains_key(&key) {
            return;
        }
        self.instance_map.remove(&key);
        self.instance_keys = Arc::new(self.instance_map.keys().map(|k| k.clone()).collect());
        self.rebuild_hash_ring();
    }

    pub fn select_instance(
        &mut self,
        router: &RouterStrategy,
        job_id: u64,
    ) -> InstanceAddrSelectResult {
        if self.instance_keys.is_empty() {
            return InstanceAddrSelectResult::Empty;
        }
        //TODO 过滤掉不可用的实例后再做选择

        match router {
            RouterStrategy::First => {
                InstanceAddrSelectResult::Selected(self.instance_keys.first().unwrap().clone())
            }
            RouterStrategy::Last => {
                InstanceAddrSelectResult::Selected(self.instance_keys.last().unwrap().clone())
            }
            RouterStrategy::RoundRobin => {
                let index = self.round_robin_index % self.instance_keys.len();
                self.round_robin_index += 1;
                InstanceAddrSelectResult::Selected(self.instance_keys[index].clone())
            }
            RouterStrategy::Random => {
                let mut rng = rand::thread_rng();
                let selected = self.instance_keys.choose(&mut rng).unwrap();
                InstanceAddrSelectResult::Selected(selected.clone())
            }
            RouterStrategy::ConsistentHash => {
                if let Some(selected) = self.hash_ring.get_instance(job_id) {
                    InstanceAddrSelectResult::Selected(selected)
                } else {
                    InstanceAddrSelectResult::Empty
                }
            }
            RouterStrategy::ShardingBroadcast => {
                InstanceAddrSelectResult::ALL(self.instance_keys.clone())
            }
        }
    }
}
