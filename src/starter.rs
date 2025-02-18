use crate::app::core::AppManager;
use crate::common::app_config::AppConfig;
use crate::common::share_data::ShareData;
use crate::job::core::JobManager;
use crate::schedule::core::ScheduleManager;
use crate::sequence::SequenceManager;
use crate::task::core::TaskManager;
use actix::Actor;
use bean_factory::{BeanDefinition, BeanFactory, FactoryData};
use std::sync::Arc;
use crate::webhook::core::WebHookManager;

pub async fn config_factory(app_config: Arc<AppConfig>) -> anyhow::Result<FactoryData> {
    std::fs::create_dir_all(app_config.local_db_dir.as_str())?;
    //let base_path = Arc::new(app_config.local_db_dir.clone());
    let factory = BeanFactory::new();
    factory.register(BeanDefinition::from_obj(app_config.clone()));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        AppManager::new().start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        JobManager::new().start(),
    ));
    factory.register(BeanDefinition::actor_from_obj(
        SequenceManager::new().start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        ScheduleManager::new(app_config.gmt_fixed_offset_hours.map(|v| v * 60 * 60)).start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        TaskManager::new(app_config.clone()).start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        WebHookManager::new().start(),
    ));
    Ok(factory.init().await)
}

pub fn build_share_data(factory_data: FactoryData) -> anyhow::Result<Arc<ShareData>> {
    let app_config: Arc<AppConfig> = factory_data.get_bean().unwrap();
    let app_manager = factory_data.get_actor().unwrap();
    let job_manager = factory_data.get_actor().unwrap();
    let sequence_manager = factory_data.get_actor().unwrap();
    let schedule_manager = factory_data.get_actor().unwrap();
    let task_manager = factory_data.get_actor().unwrap();
    let app_data = Arc::new(ShareData {
        app_config,
        app_manager,
        job_manager,
        sequence_manager,
        schedule_manager,
        task_manager,
        factory_data,
    });
    Ok(app_data)
}
