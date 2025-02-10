use crate::schedule::model::{JobRunState, TriggerInfo};
use actix::prelude::*;
use inner_mem_cache::TimeoutSet;
use std::collections::HashMap;

pub struct ScheduleManager {
    job_run_state: HashMap<u64, JobRunState>,
    active_time_set: TimeoutSet<TriggerInfo>,
}

impl Actor for ScheduleManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("ScheduleManager started")
    }
}

impl ScheduleManager {
    pub fn new() -> Self {
        ScheduleManager {
            job_run_state: HashMap::new(),
            active_time_set: TimeoutSet::new(),
        }
    }
}
