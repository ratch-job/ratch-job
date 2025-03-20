use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatusType {
    Init,
    Running,
    Error,
    Success,
}

impl Default for TaskStatusType {
    fn default() -> Self {
        TaskStatusType::Init
    }
}

impl TaskStatusType {
    pub fn from_str(status_type: &str) -> TaskStatusType {
        match status_type {
            "INIT" => TaskStatusType::Init,
            "RUNNING" => TaskStatusType::Running,
            "ERROR" => TaskStatusType::Error,
            "SUCCESS" => TaskStatusType::Success,
            _ => TaskStatusType::Init,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            TaskStatusType::Init => "INIT",
            TaskStatusType::Running => "RUNNING",
            TaskStatusType::Error => "ERROR",
            TaskStatusType::Success => "SUCCESS",
        }
    }
}
