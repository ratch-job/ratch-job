use crate::job::model::job::JobTaskLogQueryParam;
use crate::task::model::actor_model::{TaskHistoryManagerReq, TaskHistoryManagerResult};
use crate::task::model::task::JobTaskInfo;
use actix::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct TaskHistoryManager {
    task_latest_history_map: BTreeMap<u64, Arc<JobTaskInfo>>,
    latest_limit_count: usize,
}

impl TaskHistoryManager {
    pub fn new() -> Self {
        TaskHistoryManager {
            task_latest_history_map: BTreeMap::new(),
            latest_limit_count: 10000,
        }
    }

    pub fn update_task_log(&mut self, new_task_log: Arc<JobTaskInfo>) {
        if let Some(task_log) = self.task_latest_history_map.get_mut(&new_task_log.task_id) {
            *task_log = new_task_log;
        } else {
            self.task_latest_history_map
                .insert(new_task_log.task_id, new_task_log);
            if self.task_latest_history_map.len() > self.latest_limit_count {
                self.task_latest_history_map.pop_first();
            }
        }
    }

    fn query_latest_history_task_logs(
        &self,
        query_param: &JobTaskLogQueryParam,
    ) -> (usize, Vec<Arc<JobTaskInfo>>) {
        let mut rlist = Vec::new();
        let end_index = query_param.offset + query_param.limit;
        let mut index = 0;

        for (_task_id, task_log) in self.task_latest_history_map.iter().rev() {
            if index >= query_param.offset && index < end_index {
                rlist.push(task_log.clone());
            }
            index += 1;
        }
        (index, rlist)
    }
}

impl Actor for TaskHistoryManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("TaskHistoryManager started!");
    }
}

impl Handler<TaskHistoryManagerReq> for TaskHistoryManager {
    type Result = anyhow::Result<TaskHistoryManagerResult>;

    fn handle(&mut self, msg: TaskHistoryManagerReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TaskHistoryManagerReq::UpdateTask(task) => {
                self.update_task_log(task);
                Ok(TaskHistoryManagerResult::None)
            }
            TaskHistoryManagerReq::QueryJobTaskLog(param) => {
                let (total_count, list) = self.query_latest_history_task_logs(&param);
                Ok(TaskHistoryManagerResult::JobTaskLogPageInfo(
                    total_count,
                    list,
                ))
            }
        }
    }
}
