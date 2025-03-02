use crate::task::model::task::JobTaskInfo;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct JobTaskLogGroup {
    pub task_log_map: BTreeMap<u64, Arc<JobTaskInfo>>,
}

impl JobTaskLogGroup {
    pub fn new() -> Self {
        JobTaskLogGroup {
            task_log_map: BTreeMap::new(),
        }
    }

    pub fn update_task_log(&mut self, new_task_log: Arc<JobTaskInfo>, limit_count: usize) {
        if let Some(task_log) = self.task_log_map.get_mut(&new_task_log.task_id) {
            *task_log = new_task_log;
        } else {
            self.task_log_map.insert(new_task_log.task_id, new_task_log);
            if self.task_log_map.len() > limit_count {
                self.task_log_map.pop_first();
            }
        }
    }
}
