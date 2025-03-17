use crate::app::model::AppKey;
use crate::common::hash_utils::get_hash_value;
use crate::job::model::enum_type::RouterStrategy;
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use std::sync::Arc;

pub enum InstanceAddrSelectResult {
    Fixed(Arc<String>),
    Selected(Arc<String>),
    ALL(Vec<Arc<String>>),
    Empty,
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
    pub instance_keys: Vec<Arc<String>>,
    pub round_robin_index: usize,
}

impl AppInstanceStateGroup {
    pub fn new(app_key: AppKey) -> Self {
        AppInstanceStateGroup {
            app_key,
            instance_map: HashMap::new(),
            instance_keys: Vec::new(),
            round_robin_index: 0,
        }
    }

    pub fn clean(&mut self) {
        self.round_robin_index = 0;
        self.instance_map = HashMap::new();
        self.instance_keys = Vec::new();
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
        self.instance_keys.push(key);
    }

    pub fn remove_instance(&mut self, key: Arc<String>) {
        if !self.instance_map.contains_key(&key) {
            return;
        }
        self.instance_map.remove(&key);
        self.instance_keys.retain(|k| k != &key);
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
                let hash = get_hash_value(&job_id) as usize;
                let selected = self.instance_keys[hash % self.instance_keys.len()].clone();
                InstanceAddrSelectResult::Selected(selected)
            }
            RouterStrategy::ShardingBroadcast => {
                InstanceAddrSelectResult::ALL(self.instance_keys.clone())
            }
        }
    }
}
