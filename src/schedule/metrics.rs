use crate::metrics::model::{MetricsItem, MetricsQuery};
use crate::schedule::core::ScheduleManager;
use actix::{Context, Handler};

impl Handler<MetricsQuery> for ScheduleManager {
    type Result = anyhow::Result<Vec<MetricsItem>>;

    fn handle(&mut self, _msg: MetricsQuery, _ctx: &mut Context<Self>) -> Self::Result {
        //todo
        Ok(vec![])
    }
}
