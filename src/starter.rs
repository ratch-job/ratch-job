use crate::app::core::AppManager;
use crate::common::app_config::AppConfig;
use crate::common::share_data::ShareData;
use crate::job::core::JobManager;
use actix::Actor;
use bean_factory::{BeanDefinition, BeanFactory, FactoryData};
use std::sync::Arc;

pub async fn config_factory(app_config: Arc<AppConfig>) -> anyhow::Result<FactoryData> {
    std::fs::create_dir_all(app_config.local_db_dir.as_str())?;
    //let base_path = Arc::new(app_config.local_db_dir.clone());
    let factory = BeanFactory::new();
    factory.register(BeanDefinition::from_obj(app_config.clone()));
    let app_manager = AppManager::new().start();
    factory.register(BeanDefinition::actor_from_obj(app_manager));
    let job_manager = JobManager::new().start();
    factory.register(BeanDefinition::actor_from_obj(job_manager));
    Ok(factory.init().await)
}

pub fn build_share_data(factory_data: FactoryData) -> anyhow::Result<Arc<ShareData>> {
    let app_config: Arc<AppConfig> = factory_data.get_bean().unwrap();
    let app_manager = factory_data.get_actor().unwrap();
    let job_manager = factory_data.get_actor().unwrap();
    let app_data = Arc::new(ShareData {
        app_config,
        app_manager,
        job_manager,
        factory_data,
    });
    Ok(app_data)
}
