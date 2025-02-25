use self::model::LogRecordDto;
use crate::sequence::model::SequenceRaftReq;
use async_raft_ext::raft::{Entry, EntryPayload};
use async_raft_ext::{AppData, AppDataResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod core;
pub mod log;
pub mod model;
pub mod raftapply;
pub mod raftdata;
pub mod raftindex;
pub mod raftlog;
pub mod raftsnapshot;

pub type NodeId = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientRequest {
    NodeAddr { id: u64, addr: Arc<String> },
    Members(Vec<u64>),
    //SequenceReq{ req: SequenceRaftReq }
}

impl AppData for ClientRequest {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientResponse {
    Success,
    Fail,
}

impl Default for ClientResponse {
    fn default() -> Self {
        Self::Success
    }
}

impl AppDataResponse for ClientResponse {}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error("unsafe storage error")]
    UnsafeStorageError,
}

pub struct StoreUtils;

impl StoreUtils {
    pub fn log_record_to_entry(record: LogRecordDto) -> anyhow::Result<Entry<ClientRequest>> {
        let payload: EntryPayload<ClientRequest> = serde_json::from_slice(&record.value)?;
        let entry = Entry {
            term: record.term,
            index: record.index,
            payload,
        };
        Ok(entry)
    }

    pub fn entry_to_record(entry: &Entry<ClientRequest>) -> anyhow::Result<LogRecordDto> {
        let value = serde_json::to_vec(&entry.payload)?;
        let record = LogRecordDto {
            index: entry.index,
            term: entry.term,
            value,
        };
        Ok(record)
    }
}
