use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use cron::Schedule;
use std::str::FromStr;
use std::time::SystemTime;

pub struct CronUtil;

impl CronUtil {
    pub fn check_cron_valid(cron_value: &str) -> bool {
        Schedule::from_str(cron_value).is_ok()
    }

    pub fn next_cron_time_by_timestamp(
        cron_schedule: &Schedule,
        fixed_offset: &FixedOffset,
        secs: u32,
    ) -> anyhow::Result<u32> {
        let dt = if let Some(v) = DateTime::<Utc>::from_timestamp(secs as i64, 0) {
            v.with_timezone(fixed_offset)
        } else {
            return Err(anyhow::anyhow!(
                "DateTime::from_timestamp error! secs:{}",
                secs
            ));
        };
        Self::next_cron_time(cron_schedule, &dt)
    }

    pub fn next_cron_time<T: TimeZone>(
        cron_schedule: &Schedule,
        datetime: &DateTime<T>,
    ) -> anyhow::Result<u32> {
        cron_schedule
            .after(&datetime)
            .next()
            .map(|v| v.timestamp() as u32)
            .ok_or(anyhow::anyhow!("calculate_next_cron_time error!"))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::cron_utils::CronUtil;
    use crate::common::datetime_utils::now_second_u32;
    use chrono::FixedOffset;
    use std::str::FromStr;

    #[test]
    fn test_cron_util() -> anyhow::Result<()> {
        let cron_value = "* * * * * *";
        let cron_schedule = cron::Schedule::from_str(cron_value)?;
        let now_seconds = now_second_u32();
        println!("now_seconds:{}", now_seconds);
        let next_time = CronUtil::next_cron_time_by_timestamp(
            &cron_schedule,
            &FixedOffset::east(8 * 3600),
            now_seconds,
        )?;
        println!("next_time:{}", next_time);
        let next_time = CronUtil::next_cron_time_by_timestamp(
            &cron_schedule,
            &FixedOffset::east(8 * 3600),
            next_time,
        )?;
        println!("next_time:{}", next_time);
        Ok(())
    }
}
