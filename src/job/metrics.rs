use crate::job::core::JobManager;
use crate::metrics::metrics_key::MetricsKey;
use crate::metrics::model::{MetricsItem, MetricsQuery, MetricsRecord};
use actix::{Context, Handler};

impl Handler<MetricsQuery> for JobManager {
    type Result = anyhow::Result<Vec<MetricsItem>>;

    fn handle(&mut self, _msg: MetricsQuery, _ctx: &mut Context<Self>) -> Self::Result {
        let list = vec![
            MetricsItem {
                metrics_type: MetricsKey::JobSize,
                record: MetricsRecord::Gauge(self.job_map.len() as f32),
            },
            MetricsItem {
                metrics_type: MetricsKey::JobEnableSize,
                record: MetricsRecord::Gauge(self.get_all_enable_size() as f32),
            },
        ];
        Ok(list)
    }
}
