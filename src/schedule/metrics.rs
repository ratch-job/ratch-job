use crate::metrics::metrics_key::MetricsKey;
use crate::metrics::model::{MetricsItem, MetricsQuery, MetricsRecord};
use crate::schedule::core::ScheduleManager;
use actix::{Context, Handler};

impl Handler<MetricsQuery> for ScheduleManager {
    type Result = anyhow::Result<Vec<MetricsItem>>;

    fn handle(&mut self, _msg: MetricsQuery, _ctx: &mut Context<Self>) -> Self::Result {
        let list = vec![MetricsItem {
            metrics_type: MetricsKey::TaskRunningSize,
            record: MetricsRecord::Gauge(self.running_task.len() as f32),
        }];
        Ok(list)
    }
}
