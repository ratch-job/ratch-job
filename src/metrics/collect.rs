use crate::app::core::AppManager;
use crate::job::core::JobManager;
use crate::metrics::model::{MetricsItem, MetricsQuery};
use crate::schedule::core::ScheduleManager;
use crate::task::request_actor::TaskRequestActor;
use actix::prelude::*;
use bean_factory::FactoryData;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct MetricsActorCollect {
    pub app_manager: Addr<AppManager>,
    pub job_manager: Addr<JobManager>,
    pub schedule_manager: Addr<ScheduleManager>,
    pub task_request_actor: Addr<TaskRequestActor>,
}

impl MetricsActorCollect {
    pub fn from_factory(factory_data: &FactoryData) -> Option<Arc<Self>> {
        let app_manager = if let Some(app_manager) = factory_data.get_actor() {
            app_manager
        } else {
            return None;
        };
        let job_manager = if let Some(job_manager) = factory_data.get_actor() {
            job_manager
        } else {
            return None;
        };
        let schedule_manager = if let Some(schedule_manager) = factory_data.get_actor() {
            schedule_manager
        } else {
            return None;
        };
        let task_request_actor = if let Some(task_request_actor) = factory_data.get_actor() {
            task_request_actor
        } else {
            return None;
        };
        Some(Arc::new(Self {
            app_manager,
            job_manager,
            schedule_manager,
            task_request_actor,
        }))
    }

    pub async fn peek_metrics(&self) -> anyhow::Result<Vec<MetricsItem>> {
        let mut list = vec![];
        let mut t = self.app_manager.send(MetricsQuery).await??;
        list.append(&mut t);
        let mut t = self.job_manager.send(MetricsQuery).await??;
        list.append(&mut t);
        let mut t = self.schedule_manager.send(MetricsQuery).await??;
        list.append(&mut t);
        let mut t = self.task_request_actor.send(MetricsQuery).await??;
        list.append(&mut t);
        Ok(list)
    }
}
