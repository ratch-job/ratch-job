use crate::sequence::model::{SequenceRaftReq, SequenceRaftResult};
use actix::prelude::*;
use bean_factory::bean;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SequenceDbManager {
    /// value为下一次可用id
    pub(crate) seq_info: HashMap<Arc<String>, u64>,
}

impl SequenceDbManager {
    pub fn new() -> Self {
        Self {
            seq_info: HashMap::new(),
        }
    }

    pub fn next_id(&mut self, key: Arc<String>) -> u64 {
        if let Some(id) = self.seq_info.get_mut(&key) {
            let old = *id;
            *id += 1;
            old
        } else {
            self.seq_info.insert(key.clone(), 1 + 1);
            1
        }
    }

    pub fn next_range(&mut self, key: Arc<String>, step: u64) -> anyhow::Result<u64> {
        if let Some(id) = self.seq_info.get_mut(&key) {
            let old = *id;
            *id += step;
            Ok(old)
        } else {
            self.seq_info.insert(key.clone(), step + 1);
            Ok(1)
        }
    }
}

impl Actor for SequenceDbManager {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("SequenceDbManager started")
    }
}

impl Handler<SequenceRaftReq> for SequenceDbManager {
    type Result = anyhow::Result<SequenceRaftResult>;

    fn handle(&mut self, msg: SequenceRaftReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SequenceRaftReq::NextId(key) => {
                let id = self.next_id(key);
                Ok(SequenceRaftResult::NextId(id))
            }
            SequenceRaftReq::NextRange(key, step) => {
                let start = self.next_range(key, step)?;
                Ok(SequenceRaftResult::NextRange { start, len: step })
            }
            SequenceRaftReq::SetId(key, id) => {
                self.seq_info.insert(key, id);
                Ok(SequenceRaftResult::None)
            }
            SequenceRaftReq::RemoveId(key) => {
                self.seq_info.remove(&key);
                Ok(SequenceRaftResult::None)
            }
        }
    }
}
