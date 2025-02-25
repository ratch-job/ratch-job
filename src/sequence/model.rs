use actix::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "anyhow::Result<SequenceRaftResult>")]
pub enum SequenceRaftReq {
    GetNextId(Arc<String>),
    GetNextRange(Arc<String>, u64),
    SetId(Arc<String>, u64),
    RemoveId(Arc<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SequenceRaftResult {
    NextId(u64),
    NextRange { start: u64, len: u64 },
    None,
}
