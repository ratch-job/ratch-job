use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::job::model::job::{JobInfo, JobKey, JobParam};
use actix::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct JobManager {
    job_map: HashMap<u64, Arc<JobInfo>>,
    key_to_id: HashMap<JobKey, u64>,
}

impl JobManager {
    pub fn new() -> Self {
        JobManager {
            job_map: HashMap::new(),
            key_to_id: HashMap::new(),
        }
    }

    fn update_job(&mut self, job_param: JobParam) {
        if let Some(job_info) = self.job_map.get(&job_param.id) {
            let old_key = job_info.get_key();
            let mut new_job = job_info.as_ref().clone();
            new_job.update_param(job_param);
            let new_key = job_info.get_key();
            if old_key != new_key {
                self.key_to_id.remove(&old_key);
                self.key_to_id.insert(new_key, job_info.id);
            }
            self.job_map.insert(job_info.id, Arc::new(new_job));
        } else {
            let job_info: JobInfo = job_param.into();
            if !job_info.handle_name.is_empty() {
                let key = job_info.get_key();
                self.key_to_id.insert(key, job_info.id);
            }
            self.job_map.insert(job_info.id, Arc::new(job_info));
        }
    }

    fn remove_job(&mut self, id: u64) {
        if let Some(job_info) = self.job_map.remove(&id) {
            let key = job_info.get_key();
            self.key_to_id.remove(&key);
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
            JobManagerReq::UpdateJob(job_param) => {
                if job_param.id == 0 {
                    return Err(anyhow::anyhow!("UpdateJob JobParam.id==0 is invalid!"));
                }
                self.update_job(job_param);
            }
            JobManagerReq::Remove(id) => {
                self.remove_job(id);
            }
            JobManagerReq::GetJob(id) => {
                if let Some(job_info) = self.job_map.get(&id) {
                    return Ok(JobManagerResult::JobInfo(job_info.clone()));
                }
            }
            JobManagerReq::GetJobByKey(key) => {
                if let Some(id) = self.key_to_id.get(&key) {
                    if let Some(job_info) = self.job_map.get(id) {
                        return Ok(JobManagerResult::JobInfo(job_info.clone()));
                    }
                }
            }
        }
        Ok(JobManagerResult::None)
    }
}
