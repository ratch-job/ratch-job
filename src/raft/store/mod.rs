use self::model::LogRecordDto;
use crate::app::model::{AppManagerRaftReq, AppManagerRaftResult};
use crate::cache::actor_model::{CacheManagerRaftReq, CacheManagerRaftResult};
use crate::job::model::actor_model::{JobManagerRaftReq, JobManagerRaftResult};
use crate::schedule::model::actor_model::{ScheduleManagerRaftReq, ScheduleManagerRaftResult};
use crate::sequence::model::{SequenceRaftReq, SequenceRaftResult};
use crate::user::actor_model::{UserManagerRaftReq, UserManagerRaftResult};
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
    AppReq { req: AppManagerRaftReq },
    SequenceReq { req: SequenceRaftReq },
    JobReq { req: JobManagerRaftReq },
    ScheduleReq { req: ScheduleManagerRaftReq },
    CacheReq { req: CacheManagerRaftReq },
    UserReq { req: UserManagerRaftReq },
}

impl AppData for ClientRequest {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientResponse {
    Success,
    Fail,
    AppResp {
        resp: AppManagerRaftResult,
    },
    SequenceResp {
        resp: SequenceRaftResult,
    },
    JobResp {
        resp: JobManagerRaftResult,
    },
    ///前期写错，后面改ScheduleResp;之后找合适时机删除
    #[deprecated]
    ScheduleReq {
        resp: ScheduleManagerRaftResult,
    },
    ScheduleResp {
        resp: ScheduleManagerRaftResult,
    },
    CacheResp {
        resp: CacheManagerRaftResult,
    },
    UserResp {
        resp: UserManagerRaftResult,
    },
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
