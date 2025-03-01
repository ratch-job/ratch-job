use crate::job::core::JobManager;
use crate::sequence::core::SequenceDbManager;
use actix::prelude::*;

#[derive(Clone)]
pub struct RaftDataWrap {
    pub sequence_db: Addr<SequenceDbManager>,
    pub job_manager: Addr<JobManager>,
}
