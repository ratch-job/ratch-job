use crate::schedule::model::task::JobTaskInfo;
use std::collections::HashMap;

pub struct JobTaskInstanceManager {
    task_instance_map: HashMap<u64, JobTaskInfo>,
}
