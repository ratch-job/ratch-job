use cron::Schedule;
use std::str::FromStr;

pub struct CronUtil;

impl CronUtil {
    pub fn check_cron_valid(cron_value: &str) -> bool {
        Schedule::from_str(cron_value).is_ok()
    }
}
