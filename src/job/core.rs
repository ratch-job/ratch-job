use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::job::model::job::{JobInfo, JobKey};
use actix::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct JobManager {
    pub(crate) job_map: HashMap<JobKey, Arc<JobInfo>>,
}

impl JobManager {
    pub fn new() -> Self {
        JobManager {
            job_map: HashMap::new(),
        }
    }
}

impl Actor for JobManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("JobManager started");
    }
}

impl Handler<JobManagerReq> for JobManager {
    type Result = anyhow::Result<JobManagerResult>;

    fn handle(&mut self, msg: JobManagerReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            JobManagerReq::UpdateJob(_) => {}
            JobManagerReq::Remove(_) => {}
        }
        Ok(JobManagerResult::None)
    }
}
