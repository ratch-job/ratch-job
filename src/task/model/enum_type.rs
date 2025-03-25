use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatusType {
    Init,
    Running,
    Fail,
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
            "FAIL" => TaskStatusType::Fail,
            "SUCCESS" => TaskStatusType::Success,
            _ => TaskStatusType::Init,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            TaskStatusType::Init => "INIT",
            TaskStatusType::Running => "RUNNING",
            TaskStatusType::Fail => "FAIL",
            TaskStatusType::Success => "SUCCESS",
        }
    }

    pub fn is_fail(&self) -> bool {
        match self {
            TaskStatusType::Init => false,
            TaskStatusType::Running => false,
            TaskStatusType::Fail => true,
            TaskStatusType::Success => false,
        }
    }

    pub fn is_finish(&self) -> bool {
        match self {
            TaskStatusType::Init => false,
            TaskStatusType::Running => false,
            TaskStatusType::Fail => true,
            TaskStatusType::Success => true,
        }
    }

    pub fn is_running(&self) -> bool {
        match self {
            TaskStatusType::Init => false,
            TaskStatusType::Running => true,
            TaskStatusType::Fail => false,
            TaskStatusType::Success => false,
        }
    }
}
