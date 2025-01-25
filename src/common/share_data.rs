use crate::common::app_config::AppConfig;
use bean_factory::FactoryData;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ShareData {
    pub app_config: Arc<AppConfig>,
    pub factory_data: FactoryData,
}
