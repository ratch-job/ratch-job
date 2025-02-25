pub mod code;
pub mod model;

use crate::common::sequence_utils::SimpleSequence;
use crate::sequence::model::{SequenceRaftReq, SequenceRaftResult};
use actix::prelude::*;
use serde::{Deserialize, Serialize};
/// 获取顺序递增的id功能
use std::collections::HashMap;
use std::sync::Arc;

///
/// 序号管理器
/// TODO: 先只是基于内存的简单实现，后续为基于raft做持久化
#[derive(Clone, Debug)]
pub struct SequenceManager {
    pub(crate) seq_info: HashMap<Arc<String>, SimpleSequence>,
}

impl SequenceManager {
    pub fn new() -> Self {
        SequenceManager {
            seq_info: HashMap::new(),
        }
    }

    fn next_id(&mut self, key: Arc<String>) -> u64 {
        if let Some(v) = self.seq_info.get_mut(&key) {
            v.next_id()
        } else {
            let mut seq = SimpleSequence::new(0, 100);
            let id = seq.next_id();
            self.seq_info.insert(key.clone(), seq);
            id
        }
    }
}

impl Actor for SequenceManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("SequenceManager start")
    }
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "anyhow::Result<SequenceResult>")]
pub enum SequenceRequest {
    GetNextId(Arc<String>),
}

pub enum SequenceResult {
    NextId(u64),
    None,
}

impl Handler<SequenceRequest> for SequenceManager {
    type Result = anyhow::Result<SequenceResult>;

    fn handle(&mut self, msg: SequenceRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SequenceRequest::GetNextId(key) => {
                let v = self.next_id(key);
                Ok(SequenceResult::NextId(v))
            }
        }
    }
}
