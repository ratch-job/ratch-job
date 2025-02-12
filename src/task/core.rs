use crate::app::model::AppKey;
use crate::task::model::actor_model::{TaskManagerReq, TaskManagerResult};
use crate::task::model::app_instance::AppInstanceStateGroup;
use crate::task::model::task::JobTaskInfo;
use actix::prelude::*;
use std::collections::HashMap;

pub struct TaskManager {
    task_instance_map: HashMap<u64, JobTaskInfo>,
    app_instance_group: HashMap<AppKey, AppInstanceStateGroup>,
}

impl Actor for TaskManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("TaskManager started")
    }
}

impl Handler<TaskManagerReq> for TaskManager {
    type Result = ResponseActFuture<Self, anyhow::Result<TaskManagerResult>>;

    fn handle(&mut self, msg: TaskManagerReq, ctx: &mut Self::Context) -> Self::Result {
        let fut = async move { Ok(TaskManagerResult::None) }
            .into_actor(self)
            .map(|r, _, _| r);
        Box::pin(fut)
    }
}
