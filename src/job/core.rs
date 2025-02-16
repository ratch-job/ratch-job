use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::job::model::job::{JobInfo, JobKey, JobParam, JobWrap};
use crate::schedule::core::ScheduleManager;
use crate::schedule::model::actor_model::ScheduleManagerReq;
use crate::task::model::task::JobTaskInfo;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use std::collections::HashMap;
use std::sync::Arc;

#[bean(inject)]
pub struct JobManager {
    job_map: HashMap<u64, JobWrap>,
    schedule_manager: Option<Addr<ScheduleManager>>,
    job_task_log_limit: usize,
}

impl JobManager {
    pub fn new() -> Self {
        JobManager {
            job_map: HashMap::new(),
            schedule_manager: None,
            job_task_log_limit: 200,
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
        let value = Arc::new(job_info);
        self.job_map.insert(value.id, JobWrap::new(value.clone()));
        if let Some(schedule_manager) = self.schedule_manager.as_ref() {
            schedule_manager.do_send(ScheduleManagerReq::UpdateJob(value.clone()));
        }
        Ok(value)
    }

    fn update_job(&mut self, job_param: JobParam) -> anyhow::Result<()> {
        let id = job_param.id.unwrap_or_default();
        if id == 0 {
            return Err(anyhow::anyhow!("UpdateJob JobParam.id==0 is invalid!"));
        }
        if let Some(job_wrap) = self.job_map.get(&id) {
            let job_info = &job_wrap.job;
            let mut new_job = job_info.as_ref().clone();
            new_job.update_param(job_param);
            job_info.check_valid()?;
            let value = Arc::new(new_job);
            self.job_map
                .insert(job_info.id, JobWrap::new(value.clone()));
            if let Some(schedule_manager) = self.schedule_manager.as_ref() {
                schedule_manager.do_send(ScheduleManagerReq::UpdateJob(value.clone()));
            }
        } else {
            return Err(anyhow::anyhow!("UpdateJob,Nonexistent Job"));
        }
        Ok(())
    }

    fn remove_job(&mut self, id: u64) {
        self.job_map.remove(&id);
        if let Some(schedule_manager) = self.schedule_manager.as_ref() {
            schedule_manager.do_send(ScheduleManagerReq::RemoveJob(id));
        }
    }

    fn update_job_task(&mut self, task_log: Arc<JobTaskInfo>) {
        if let Some(job_wrap) = self.job_map.get_mut(&task_log.job_id) {
            job_wrap.update_task_log(task_log, self.job_task_log_limit);
        }
    }
}

impl Actor for JobManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("JobManager started");
    }
}

impl Inject for JobManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.schedule_manager = factory_data.get_actor();
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
                if let Some(job_wrap) = self.job_map.get(&id) {
                    return Ok(JobManagerResult::JobInfo(job_wrap.job.clone()));
                }
            }
            JobManagerReq::UpdateTask(task_info) => {
                self.update_job_task(task_info);
            }
        }
        Ok(JobManagerResult::None)
    }
}
