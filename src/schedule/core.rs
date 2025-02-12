use crate::common::datetime_utils::{
    get_datetime_by_second, get_datetime_millis, get_local_offset, now_millis, now_second_u32,
};
use crate::job::model::enum_type::ScheduleType;
use crate::job::model::job::JobInfo;
use crate::schedule::model::actor_model::{ScheduleManagerReq, ScheduleManagerResult};
use crate::schedule::model::{JobRunState, TriggerInfo};
use actix::prelude::*;
use actix_web::cookie::time::macros::datetime;
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Offset, TimeZone, Utc};
use inner_mem_cache::TimeoutSet;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ScheduleManager {
    job_run_state: HashMap<u64, JobRunState>,
    active_time_set: TimeoutSet<TriggerInfo>,
    fixed_offset: FixedOffset,
}

impl Actor for ScheduleManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("ScheduleManager started");
        self.heartbeat(ctx);
    }
}

impl ScheduleManager {
    pub fn new(offset_seconds: Option<i32>) -> Self {
        let fixed_offset = if let Some(offset_value) = offset_seconds {
            FixedOffset::east_opt(offset_value).unwrap_or(get_local_offset())
        } else {
            get_local_offset()
        };
        ScheduleManager {
            job_run_state: HashMap::new(),
            active_time_set: TimeoutSet::new(),
            fixed_offset,
        }
    }

    fn active_job(&mut self, job_id: u64, time: u32, version: u32) {
        self.active_time_set
            .add(time as u64, TriggerInfo::new(job_id, time, version));
    }

    fn update_job_trigger_time(&mut self, job_id: u64, last_time:u32, next_time:u32) {
        if let Some(job)= self.job_run_state.get_mut(&job_id) {
            job.pre_trigger_time = last_time;
            job.next_trigger_time=next_time;
        }
    }

    fn update_job(&mut self, job_info: Arc<JobInfo>) {
        let job_id = job_info.id;
        if job_info.schedule_type == ScheduleType::None {
            self.job_run_state.remove(&job_id);
            return;
        }
        let mut active_job_param = None;
        if let Some(job_run_state) = self.job_run_state.get_mut(&job_id) {
            let change_schedule = job_run_state.update_job(job_info);
            if change_schedule {
                if let Some(now_datetime) =
                    get_datetime_by_second(now_second_u32() - 1, &self.fixed_offset)
                {
                    let next_trigger_time =
                        job_run_state.calculate_first_trigger_time(&now_datetime);
                    job_run_state.next_trigger_time = next_trigger_time;
                    active_job_param = Some((next_trigger_time, job_run_state.version))
                }
            }
        } else {
            let mut job_run_state = JobRunState::new(job_info);
            if let Some(now_datetime) = get_datetime_by_second(now_second_u32(), &self.fixed_offset)
            {
                let next_trigger_time = job_run_state.calculate_first_trigger_time(&now_datetime);
                job_run_state.next_trigger_time = next_trigger_time;
                active_job_param = Some((next_trigger_time, job_run_state.version))
            }
            self.job_run_state.insert(job_id, job_run_state);
        }
        if let Some((next_trigger_time, version)) = active_job_param {
            self.active_job(job_id, next_trigger_time, version);
        }
    }

    fn remove_job(&mut self, job_id: u64) {
        self.job_run_state.remove(&job_id);
    }

    fn trigger_job(&mut self, seconds: u32) {
        let date_time = get_datetime_by_second(seconds, &self.fixed_offset).unwrap();
        for item in self.active_time_set.timeout(seconds as u64) {
            if let Some(job) = self.job_run_state.get(&item.job_id) {
                if job.version != item.version {
                    //ignore
                    log::info!("job version change ignore,id:{}", &item.job_id);
                    continue;
                }
                log::info!(
                    "prepare job,id:{},run_mode:{:?},handler_name:{}",
                    &job.id,
                    &job.source_job.run_mode,
                    &job.source_job.handle_name
                );
                //TODO trigger job

                let next_trigger_time = job.calculate_next_trigger_time(&date_time);
                if next_trigger_time > 0 {
                    self.active_job(item.job_id, next_trigger_time, job.version);
                    self.update_job_trigger_time(item.job_id,item.trigger_time,next_trigger_time);
                } else {
                    log::info!("job next trigger is none,id:{}", &item.job_id);
                }
            } else {
                log::info!("job not exist,id:{}", &item.job_id);
            }
        }
    }
    fn heartbeat(&mut self, ctx: &mut Context<Self>) {
        self.trigger_job(now_second_u32());
        let later_millis = 1000 - now_millis() % 1000;
        ctx.run_later(
            std::time::Duration::from_millis(later_millis),
            move |act, ctx| {
                act.heartbeat(ctx);
            },
        );
    }
}

impl Handler<ScheduleManagerReq> for ScheduleManager {
    type Result = anyhow::Result<ScheduleManagerResult>;

    fn handle(&mut self, msg: ScheduleManagerReq, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            ScheduleManagerReq::UpdateJob(job) => {
                log::info!("ScheduleManagerReq::UpdateJob,job_id:{}", &job.id);
                self.update_job(job);
            }
            ScheduleManagerReq::RemoveJob(job_id) => {
                self.remove_job(job_id);
            }
        }
        Ok(ScheduleManagerResult::None)
    }
}
