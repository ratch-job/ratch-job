use crate::app::app_index::{AppIndex, AppQueryParam};
use crate::app::model::{
    AppInfo, AppInfoDto, AppInstance, AppKey, AppManagerReq, AppManagerResult, AppParam,
    RegisterType,
};
use crate::common::constant::EMPTY_ARC_STR;
use crate::common::datetime_utils::now_millis;
use crate::task::core::TaskManager;
use crate::task::model::actor_model::TaskManagerReq;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[bean(inject)]
pub struct AppManager {
    pub(crate) app_map: HashMap<AppKey, AppInfo>,
    task_manager: Option<Addr<TaskManager>>,
    app_index: AppIndex,
    instance_timeout: u32,
}

impl AppManager {
    pub fn new() -> Self {
        AppManager {
            app_map: HashMap::new(),
            task_manager: None,
            app_index: AppIndex::new(),
            instance_timeout: 180,
        }
    }

    fn update_app(&mut self, app_param: AppParam) {
        let key = AppKey::new(app_param.name.clone(), app_param.namespace.clone());
        if let Some(app_info) = self.app_map.get_mut(&key) {
            if let Some(label) = app_param.label {
                app_info.label = label;
            }
            if let Some(register_type) = app_param.register_type {
                app_info.register_type = register_type;
            }
            Self::set_app_instance_addrs(app_info, app_param.instance_addrs);
        } else {
            self.app_index
                .insert(key.namespace.clone(), key.name.clone());
            let mut app_info = AppInfo::new(
                app_param.name.clone(),
                app_param.namespace.clone(),
                app_param.label.unwrap_or(EMPTY_ARC_STR.clone()),
                app_param.register_type.unwrap_or(RegisterType::Auto),
            );
            Self::set_app_instance_addrs(&mut app_info, app_param.instance_addrs);
            self.app_map.insert(key, app_info);
        }
    }

    fn set_app_instance_addrs(app_info: &mut AppInfo, instance_addrs: Option<Vec<Arc<String>>>) {
        if let Some(instance_addrs) = instance_addrs {
            for addr in instance_addrs {
                if let Some(instance) = app_info.instance_map.get_mut(&addr) {
                    instance.last_modified_millis = now_millis();
                    instance.enable = true;
                    instance.healthy = true;
                } else {
                    app_info
                        .instance_map
                        .insert(addr.clone(), AppInstance::new(addr));
                }
            }
        }
    }

    fn register_app_instance(&mut self, key: AppKey, instance_key: Arc<String>) {
        if let Some(task_manager) = self.task_manager.as_ref() {
            task_manager.do_send(TaskManagerReq::AddAppInstance(
                key.clone(),
                instance_key.clone(),
            ));
        }
        if let Some(app_info) = self.app_map.get_mut(&key) {
            if let Some(instance) = app_info.instance_map.get_mut(&instance_key) {
                instance.last_modified_millis = now_millis();
                instance.enable = true;
                instance.healthy = true;
            } else {
                app_info
                    .instance_map
                    .insert(instance_key.clone(), AppInstance::new(instance_key));
            }
        } else {
            self.app_index
                .insert(key.namespace.clone(), key.name.clone());
            let mut app_info = AppInfo::new(
                key.name.clone(),
                key.namespace.clone(),
                EMPTY_ARC_STR.clone(),
                RegisterType::Auto,
            );
            app_info
                .instance_map
                .insert(instance_key.clone(), AppInstance::new(instance_key));
            self.app_map.insert(key, app_info);
        }
    }

    fn check_instance_timeout(&mut self, now: u64, addr_app_manager: Addr<AppManager>) {
        for (app_key, app) in &mut self.app_map {
            //TODO 目前这个遍历的方式运算复杂度是O(n),可以使用BTreeMap优化为O(log(n))
            for (addr, instance) in &mut app.instance_map {
                let start = instance.last_modified_millis;
                if now - start > self.instance_timeout as u64 * 1000u64 {
                    addr_app_manager.do_send(AppManagerReq::UnregisterAppInstance(
                        app_key.clone(),
                        addr.clone(),
                    ));
                }
            }
        }
    }

    fn heartbeat(&mut self, ctx: &mut Context<Self>) {
        ctx.run_later(std::time::Duration::from_secs(10), move |act, ctx| {
            let addr = ctx.address();
            act.check_instance_timeout(now_millis(), addr);
            act.heartbeat(ctx);
        });
    }

    fn unregister_app_instance(&mut self, key: AppKey, instance_key: Arc<String>) {
        log::info!("unregister_app_instance, {:?} {:?}", key, instance_key);
        if let Some(task_manager) = self.task_manager.as_ref() {
            task_manager.do_send(TaskManagerReq::RemoveAppInstance(
                key.clone(),
                instance_key.clone(),
            ));
        }
        if let Some(app_info) = self.app_map.get_mut(&key) {
            app_info.instance_map.remove(&instance_key);
        }
    }

    fn get_app_instance_addrs(&self, key: AppKey) -> Arc<Vec<Arc<String>>> {
        if let Some(app_info) = self.app_map.get(&key) {
            let mut addrs = Vec::new();
            for (_, instance) in app_info.instance_map.iter() {
                if instance.enable && instance.healthy {
                    addrs.push(instance.addr.clone());
                }
            }
            return Arc::new(addrs);
        }
        Arc::new(Vec::new())
    }

    fn query_page_info(&self, query_param: &AppQueryParam) -> (usize, Vec<AppInfoDto>) {
        let (size, list) = self.app_index.query(query_param);
        if size == 0 {
            return (0, Vec::new());
        }
        let mut info_list = Vec::with_capacity(size);
        for key in &list {
            if let Some(app_info) = self.app_map.get(key) {
                info_list.push(AppInfoDto::new_from(app_info, false));
            }
        }
        (size, info_list)
    }

    fn get_app_info(&self, key: &AppKey, with_addrs: bool) -> Option<AppInfoDto> {
        self.app_map
            .get(key)
            .map(|v| AppInfoDto::new_from(v, with_addrs))
    }
}

impl Actor for AppManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        log::info!("AppManager started");
        self.heartbeat(ctx);
    }
}

impl Inject for AppManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.task_manager = factory_data.get_actor();
    }
}

impl Handler<AppManagerReq> for AppManager {
    type Result = anyhow::Result<AppManagerResult>;

    fn handle(&mut self, msg: AppManagerReq, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            AppManagerReq::RegisterAppInstance(key, addr) => {
                self.register_app_instance(key, addr);
            }
            AppManagerReq::UpdateApp(param) => {
                self.update_app(param);
            }
            AppManagerReq::RemoveApp(key) => {
                self.app_map.remove(&key);
            }
            AppManagerReq::GetApp(key) => {
                return Ok(AppManagerResult::AppInfo(self.get_app_info(&key, true)));
            }
            AppManagerReq::UnregisterAppInstance(key, addr) => {
                self.unregister_app_instance(key, addr);
            }
            AppManagerReq::GetAppInstanceAddrs(key) => {
                let addrs = self.get_app_instance_addrs(key);
                return Ok(AppManagerResult::InstanceAddrs(addrs));
            }
            AppManagerReq::QueryApp(param) => {
                let (size, list) = self.query_page_info(&param);
                return Ok(AppManagerResult::AppPageInfo(size, list));
            }
        }
        Ok(AppManagerResult::None)
    }
}
