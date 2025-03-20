use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScheduleType {
    Cron,
    Interval,
    //执行后延时，暂时不增加
    //Delay,
    None,
}

impl Default for ScheduleType {
    fn default() -> Self {
        ScheduleType::None
    }
}

impl ScheduleType {
    pub fn from_str(glue_type: &str) -> ScheduleType {
        match glue_type {
            "CRON" => ScheduleType::Cron,
            "INTERVAL" => ScheduleType::Interval,
            //"DELAY" => ScheduleType::Delay,
            _ => ScheduleType::None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            ScheduleType::Cron => "CRON",
            ScheduleType::Interval => "INTERVAL",
            //ScheduleType::Delay => "DELAY",
            ScheduleType::None => "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobRunMode {
    Bean,
    GlueGroovy,
    GlueShell,
    GluePython,
    GluePhp,
    GlueNodejs,
    GluePowerShell,
}

impl Default for JobRunMode {
    fn default() -> Self {
        JobRunMode::Bean
    }
}

impl JobRunMode {
    pub fn from_str(glue_type: &str) -> Option<JobRunMode> {
        match glue_type {
            "BEAN" => Some(JobRunMode::Bean),
            "GLUE_GROOVY" => Some(JobRunMode::GlueGroovy),
            "GLUE_SHELL" => Some(JobRunMode::GlueShell),
            "GLUE_PYTHON" => Some(JobRunMode::GluePython),
            "GLUE_PHP" => Some(JobRunMode::GluePhp),
            "GLUE_NODEJS" => Some(JobRunMode::GlueNodejs),
            "GLUE_POWERSHELL" => Some(JobRunMode::GluePowerShell),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            JobRunMode::Bean => "BEAN",
            JobRunMode::GlueGroovy => "GLUE_GROOVY",
            JobRunMode::GlueShell => "GLUE_SHELL",
            JobRunMode::GluePython => "GLUE_PYTHON",
            JobRunMode::GluePhp => "GLUE_PHP",
            JobRunMode::GlueNodejs => "GLUE_NODEJS",
            JobRunMode::GluePowerShell => "GLUE_POWERSHELL",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RouterStrategy {
    /// 第一个
    First,
    /// 最后一个
    Last,
    /// 轮询
    RoundRobin,
    /// 随机
    Random,
    /// 一致性哈希
    ConsistentHash,
    /// 分片广播
    ShardingBroadcast,
}

impl Default for RouterStrategy {
    fn default() -> Self {
        RouterStrategy::RoundRobin
    }
}

impl RouterStrategy {
    pub fn from_str(glue_type: &str) -> Option<RouterStrategy> {
        match glue_type {
            "FIRST" => Some(RouterStrategy::First),
            "LAST" => Some(RouterStrategy::Last),
            "ROUND_ROBIN" => Some(RouterStrategy::RoundRobin),
            "RANDOM" => Some(RouterStrategy::Random),
            "CONSISTENT_HASH" => Some(RouterStrategy::ConsistentHash),
            "SHARDING_BROADCAST" => Some(RouterStrategy::ShardingBroadcast),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            RouterStrategy::First => "FIRST",
            RouterStrategy::Last => "LAST",
            RouterStrategy::RoundRobin => "ROUND_ROBIN",
            RouterStrategy::Random => "RANDOM",
            RouterStrategy::ConsistentHash => "CONSISTENT_HASH",
            RouterStrategy::ShardingBroadcast => "SHARDING_BROADCAST",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PastDueStrategy {
    Default,
    Ignore,
    Execute,
}

impl Default for PastDueStrategy {
    fn default() -> Self {
        PastDueStrategy::Default
    }
}

impl PastDueStrategy {
    pub fn from_str(glue_type: &str) -> PastDueStrategy {
        match glue_type {
            "DEFAULT" => PastDueStrategy::Default,
            "IGNORE" => PastDueStrategy::Ignore,
            "EXECUTE" => PastDueStrategy::Execute,
            _ => PastDueStrategy::Default,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            PastDueStrategy::Default => "DEFAULT",
            PastDueStrategy::Ignore => "IGNORE",
            PastDueStrategy::Execute => "EXECUTE",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutorBlockStrategy {
    SerialExecution,
    DiscardLater,
    CoverEarly,
    Other,
}

impl Default for ExecutorBlockStrategy {
    fn default() -> Self {
        ExecutorBlockStrategy::SerialExecution
    }
}

impl ExecutorBlockStrategy {
    pub fn from_str(s: &str) -> ExecutorBlockStrategy {
        match s {
            "SERIAL_EXECUTION" => ExecutorBlockStrategy::SerialExecution,
            "DISCARD_LATER" => ExecutorBlockStrategy::DiscardLater,
            "COVER_EARLY" => ExecutorBlockStrategy::CoverEarly,
            _ => ExecutorBlockStrategy::Other,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            ExecutorBlockStrategy::SerialExecution => "SERIAL_EXECUTION",
            ExecutorBlockStrategy::DiscardLater => "DISCARD_LATER",
            ExecutorBlockStrategy::CoverEarly => "COVER_EARLY",
            ExecutorBlockStrategy::Other => "OTHER",
        }
    }
}
