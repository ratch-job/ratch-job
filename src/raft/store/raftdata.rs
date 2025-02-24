use actix::prelude::*;

#[derive(Clone)]
pub struct RaftDataWrap {}

impl RaftDataWrap {
    pub fn new() -> Self {
        Self {}
    }
}
