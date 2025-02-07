use crate::app::core::AppManager;
use crate::common::app_config::AppConfig;
use actix::Addr;
use bean_factory::FactoryData;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ShareData {
    pub app_config: Arc<AppConfig>,
    pub app_manager: Addr<AppManager>,
    pub factory_data: FactoryData,
}
