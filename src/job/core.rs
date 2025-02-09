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

    fn create_job(&mut self, job_param: JobParam) -> anyhow::Result<Arc<JobInfo>> {
        let id = job_param.id.unwrap_or_default();
        if id == 0 {
            return Err(anyhow::anyhow!("CreateJob JobParam.id==0 is invalid!"));
        }
        if self.job_map.contains_key(&id) {
            return Err(anyhow::anyhow!(
                "CreateJob，The job already exists and is repeatedly created"
            ));
        }
        let job_info: JobInfo = job_param.into();
        job_info.check_valid()?;
        if !job_info.handle_name.is_empty() {
            let key = job_info.get_key();
            self.key_to_id.insert(key, job_info.id);
        }
        let value = Arc::new(job_info);
        self.job_map.insert(value.id, value.clone());
        Ok(value)
    }

    fn update_job(&mut self, job_param: JobParam) -> anyhow::Result<()> {
        let id = job_param.id.unwrap_or_default();
        if id == 0 {
            return Err(anyhow::anyhow!("UpdateJob JobParam.id==0 is invalid!"));
        }
        if let Some(job_info) = self.job_map.get(&id) {
            let old_key = job_info.get_key();
            let mut new_job = job_info.as_ref().clone();
            new_job.update_param(job_param);
            job_info.check_valid()?;
            let new_key = job_info.get_key();
            if old_key != new_key {
                self.key_to_id.remove(&old_key);
                self.key_to_id.insert(new_key, job_info.id);
            }
            self.job_map.insert(job_info.id, Arc::new(new_job));
        } else {
            return Err(anyhow::anyhow!("UpdateJob,Nonexistent Job"));
        }
        Ok(())
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
            JobManagerReq::AddJob(job_param) => {
                let value = self.create_job(job_param)?;
                return Ok(JobManagerResult::JobInfo(value));
            }
            JobManagerReq::UpdateJob(job_param) => {
                self.update_job(job_param)?;
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
