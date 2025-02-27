use crate::sequence::core::SequenceDbManager;
use actix::prelude::*;

#[derive(Clone)]
pub struct RaftDataWrap {
    pub sequence_db: Addr<SequenceDbManager>,
}
