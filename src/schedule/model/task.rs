use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct JobTaskInfo {
    pub task_id: u64,
    pub job_id: u64,
    pub trigger_time: u32,
    pub trigger_message: Arc<String>,
    pub status: u32,
    pub finish_time: u32,
    pub callback_message: Arc<String>,
}
