use crate::job::core::JobManager;
use crate::schedule::core::ScheduleManager;
use crate::sequence::core::SequenceDbManager;
use actix::prelude::*;

#[derive(Clone)]
pub struct RaftDataWrap {
    pub sequence_db: Addr<SequenceDbManager>,
    pub job_manager: Addr<JobManager>,
    pub schedule_manager: Addr<ScheduleManager>,
}
