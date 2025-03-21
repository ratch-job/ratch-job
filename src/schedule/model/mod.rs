pub mod actor_model;

use crate::common::cron_utils::CronUtil;
use crate::job::model::enum_type::ScheduleType;
use crate::job::model::job::JobInfo;
use chrono::{DateTime, TimeZone};
use cron::Schedule;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct JobRunState {
    pub id: u64,
    pub schedule_type: ScheduleType,
    pub cron_value: Arc<String>,
    pub cron_schedule: Option<Schedule>,
    pub delay_second: u32,
    pub interval_second: u32,
    pub pre_trigger_time: u32,
    pub next_trigger_time: u32,
    pub next_active: bool,
    pub version: u32,
    pub route_value: u32,
    pub source_job: Arc<JobInfo>,
}

impl JobRunState {
    pub fn new(source_job: Arc<JobInfo>) -> Self {
        let cron_schedule = Schedule::from_str(source_job.cron_value.as_str()).ok();
        JobRunState {
            id: source_job.id,
            schedule_type: source_job.schedule_type.clone(),
            cron_value: source_job.cron_value.clone(),
            cron_schedule,
            delay_second: source_job.delay_second,
            interval_second: source_job.interval_second,
            pre_trigger_time: 0,
            next_trigger_time: 0,
            next_active: false,
            version: 0,
            route_value: 0,
            source_job,
        }
    }
    pub fn calculate_first_trigger_time<T: TimeZone>(&self, datetime: &DateTime<T>) -> u32 {
        match self.schedule_type {
            //ScheduleType::Delay => datetime.timestamp() as u32,
            ScheduleType::None => 0,
            _ => self.calculate_next_trigger_time(datetime),
        }
    }

    pub fn update_job(&mut self, source_job: Arc<JobInfo>) -> bool {
        let mut change_schedule = false;
        if self.schedule_type != source_job.schedule_type {
            change_schedule = true;
            self.schedule_type = source_job.schedule_type.clone();
        }
        if self.cron_value.as_str() != source_job.cron_value.as_str() {
            change_schedule = true;
            self.cron_value = source_job.cron_value.clone();
            self.cron_schedule = Schedule::from_str(source_job.cron_value.as_str()).ok();
        }
        if self.interval_second != source_job.interval_second {
            change_schedule = true;
            self.interval_second = source_job.interval_second;
        }
        if self.delay_second != source_job.delay_second {
            change_schedule = true;
            self.delay_second = source_job.delay_second;
        }
        self.source_job = source_job;
        if change_schedule {
            if self.version == u32::MAX {
                self.version = 0;
            } else {
                self.version += 1;
            }
        }
        change_schedule
    }

    pub fn calculate_next_trigger_time<T: TimeZone>(&self, datetime: &DateTime<T>) -> u32 {
        let mut result = 0;
        let timestamp_seconds = datetime.timestamp() as u32;
        match self.schedule_type {
            ScheduleType::Cron => {
                if let Some(cron_schedule) = self.cron_schedule.as_ref() {
                    if let Ok(value) = CronUtil::next_cron_time(cron_schedule, datetime) {
                        result = value;
                    }
                }
            }
            ScheduleType::Interval => {
                let remainder = ((timestamp_seconds as i32) - (self.pre_trigger_time as i32))
                    .rem_euclid(self.interval_second as i32);
                result = (timestamp_seconds as i32 - remainder) as u32 + self.interval_second;
            }
            /*
            ScheduleType::Delay => {
                result = timestamp_seconds + self.delay_second;
            }
             */
            ScheduleType::None => {}
        }
        result
    }
}

#[derive(Clone, Debug)]
pub struct TriggerInfo {
    pub job_id: u64,
    pub trigger_time: u32,
    pub version: u32,
}

impl TriggerInfo {
    pub fn new(job_id: u64, trigger_time: u32, version: u32) -> Self {
        TriggerInfo {
            job_id,
            trigger_time,
            version,
        }
    }
}
