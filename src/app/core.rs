use crate::app::model::{
    AppInfo, AppInstance, AppKey, AppManagerReq, AppManagerResult, AppParam, RegisterType,
};
use crate::common::constant::EMPTY_ARC_STR;
use crate::common::datetime_utils::now_millis;
use actix::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct AppManager {
    pub(crate) app_map: BTreeMap<AppKey, AppInfo>,
}

impl AppManager {
    pub fn new() -> Self {
        AppManager {
            app_map: BTreeMap::new(),
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
            let mut app_info = AppInfo::new(
                app_param.name.clone(),
                app_param.namespace.clone(),
                app_param.label.unwrap_or(EMPTY_ARC_STR.clone()),
                app_param.register_type.unwrap_or(RegisterType::Auto),
            );
            Self::set_app_instance_addrs(&mut app_info, app_param.instance_addrs);
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

    fn unregister_app_instance(&mut self, key: AppKey, instance_key: Arc<String>) {
        if let Some(app_info) = self.app_map.get_mut(&key) {
            app_info.instance_map.remove(&instance_key);
        }
    }
}

impl Actor for AppManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        log::info!("AppManager started");
    }
}

impl Handler<AppManagerReq> for AppManager {
    type Result = anyhow::Result<AppManagerResult>;

    fn handle(&mut self, msg: AppManagerReq, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            AppManagerReq::UpdateApp(param) => {
                self.update_app(param);
            }
            AppManagerReq::RegisterAppInstance(key, addr) => {
                self.register_app_instance(key, addr);
            }
            AppManagerReq::RemoveApp(key) => {
                self.app_map.remove(&key);
            }
            AppManagerReq::UnregisterAppInstance(key, addr) => {
                self.unregister_app_instance(key, addr);
            }
        }
        Ok(AppManagerResult::None)
    }
}
