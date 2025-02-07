use crate::app::core::AppManager;
use crate::common::app_config::AppConfig;
use crate::job::core::JobManager;
use actix::Addr;
use bean_factory::FactoryData;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ShareData {
    pub app_config: Arc<AppConfig>,
    pub app_manager: Addr<AppManager>,
    pub job_manager: Addr<JobManager>,
    pub factory_data: FactoryData,
}
