use crate::app::core::AppManager;
use crate::metrics::metrics_key::MetricsKey;
use crate::metrics::model::{MetricsItem, MetricsQuery, MetricsRecord};
use actix::{Context, Handler};

impl Handler<MetricsQuery> for AppManager {
    type Result = anyhow::Result<Vec<MetricsItem>>;

    fn handle(&mut self, _msg: MetricsQuery, _ctx: &mut Context<Self>) -> Self::Result {
        let list = vec![
            MetricsItem {
                metrics_type: MetricsKey::JobAppSize,
                record: MetricsRecord::Gauge(self.app_map.len() as f32),
            },
            MetricsItem {
                metrics_type: MetricsKey::JobAppInstanceSize,
                record: MetricsRecord::Gauge(self.get_app_instance_all_count() as f32),
            },
        ];
        Ok(list)
    }
}
