use crate::app::core::AppManager;
use crate::common::app_config::AppConfig;
use crate::job::core::JobManager;
use crate::schedule::core::ScheduleManager;
use crate::sequence::SequenceManager;
use crate::task::core::TaskManager;
use actix::Addr;
use bean_factory::FactoryData;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ShareData {
    pub app_config: Arc<AppConfig>,
    pub app_manager: Addr<AppManager>,
    pub job_manager: Addr<JobManager>,
    pub sequence_manager: Addr<SequenceManager>,
    pub schedule_manager: Addr<ScheduleManager>,
    pub task_manager: Addr<TaskManager>,
    pub factory_data: FactoryData,
}
