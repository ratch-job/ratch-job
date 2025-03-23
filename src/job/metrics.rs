use crate::job::core::JobManager;
use crate::metrics::model::{MetricsItem, MetricsQuery};
use actix::{Context, Handler};

impl Handler<MetricsQuery> for JobManager {
    type Result = anyhow::Result<Vec<MetricsItem>>;

    fn handle(&mut self, _msg: MetricsQuery, _ctx: &mut Context<Self>) -> Self::Result {
        //todo
        Ok(vec![])
    }
}
