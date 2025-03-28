use crate::metrics::metrics_key::MetricsKey;
use crate::metrics::model::{MetricsItem, MetricsQuery, MetricsRecord};
use crate::task::request_actor::TaskRequestActor;
use actix::{Context, Handler};

impl Handler<MetricsQuery> for TaskRequestActor {
    type Result = anyhow::Result<Vec<MetricsItem>>;

    fn handle(&mut self, _msg: MetricsQuery, _ctx: &mut Context<Self>) -> Self::Result {
        let list = vec![MetricsItem {
            metrics_type: MetricsKey::TaskPendingSize,
            record: MetricsRecord::Gauge(self.running_count as f32),
        }];
        Ok(list)
    }
}
