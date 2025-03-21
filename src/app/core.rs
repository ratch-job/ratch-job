use crate::app::app_index::{AppIndex, AppQueryParam};
use crate::app::model::{
    AppInfo, AppInfoDto, AppInstance, AppInstanceDto, AppInstanceKey, AppKey, AppManagerRaftReq,
    AppManagerRaftResult, AppManagerReq, AppManagerResult, AppParam, AppRouteRequest,
    AppRouteResponse, InstanceTimeoutInfo, RegisterType,
};
use crate::common::byte_utils::id_to_bin;
use crate::common::constant::{EMPTY_ARC_STR, JOB_TABLE_NAME};
use crate::common::datetime_utils::{now_millis, now_second_u32};
use crate::common::pb::data_object::AppInfoDo;
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::{RaftApplyDataRequest, RaftApplyDataResponse};
use crate::raft::store::raftsnapshot::{SnapshotWriterActor, SnapshotWriterRequest};
use crate::task::core::TaskManager;
use crate::task::model::actor_model::TaskManagerReq;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use inner_mem_cache::TimeoutSet;
use quick_protobuf::{BytesReader, Writer};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[bean(inject)]
pub struct AppManager {
    pub(crate) app_map: HashMap<AppKey, AppInfo>,
    task_manager: Option<Addr<TaskManager>>,
    app_index: AppIndex,
    app_instance_timeout: TimeoutSet<InstanceTimeoutInfo>,
    instance_timeout: u32,
}

impl AppManager {
    pub fn new() -> Self {
        AppManager {
            app_map: HashMap::new(),
            task_manager: None,
            app_index: AppIndex::new(),
            instance_timeout: 180,
            app_instance_timeout: TimeoutSet::new(),
        }
    }

    fn update_app(&mut self, app_param: AppParam) {
        let key = AppKey::new(app_param.app_name.clone(), app_param.namespace.clone());
        if let Some(app_info) = self.app_map.get_mut(&key) {
            if let Some(label) = app_param.label {
                app_info.label = label;
            }
            if let Some(register_type) = app_param.register_type {
                app_info.register_type = register_type;
            }
            Self::set_app_instance_addrs(
                app_info,
                app_param.instance_addrs,
                app_param.last_modified_time,
            );
        } else {
            self.app_index
                .insert(key.namespace.clone(), key.app_name.clone());
            let mut app_info = AppInfo::new(
                app_param.app_name.clone(),
                app_param.namespace.clone(),
                app_param.label.unwrap_or(EMPTY_ARC_STR.clone()),
                app_param.register_type.unwrap_or(RegisterType::Auto),
                false,
            );
            Self::set_app_instance_addrs(
                &mut app_info,
                app_param.instance_addrs,
                app_param.last_modified_time,
            );
            self.app_map.insert(key, app_info);
        }
    }

    fn set_app_instance_addrs(
        app_info: &mut AppInfo,
        instance_addrs: Option<Vec<Arc<String>>>,
        last_time: u32,
    ) {
        if let Some(instance_addrs) = instance_addrs {
            for addr in instance_addrs {
                if let Some(instance) = app_info.instance_map.get_mut(&addr) {
                    //instance.last_modified_time=last_time;
                    instance.enable = true;
                    instance.healthy = true;
                } else {
                    app_info
                        .instance_map
                        .insert(addr.clone(), AppInstance::new_with_time(addr, last_time));
                }
            }
        }
    }

    fn update_app_info(&mut self, app_info: AppInfo) {
        let key = app_info.build_key();
        if let Some(old_app_info) = self.app_map.get_mut(&key) {
            old_app_info.label = app_info.label;
            old_app_info.register_type = app_info.register_type;
            old_app_info.tmp = app_info.tmp;
        } else {
            self.app_index
                .insert(key.namespace.clone(), key.app_name.clone());
            self.app_map.insert(key, app_info);
        }
    }

    fn register_app_instance(
        &mut self,
        key: AppKey,
        instance_key: Arc<String>,
        last_modified_time: u32,
    ) {
        let check_time = now_second_u32() - self.instance_timeout;
        if check_time >= last_modified_time {
            return;
        }
        self.app_instance_timeout.add(
            last_modified_time as u64,
            InstanceTimeoutInfo::new(
                AppInstanceKey::new_with_app_key(key.clone(), instance_key.clone()),
                last_modified_time,
            ),
        );
        if let Some(task_manager) = self.task_manager.as_ref() {
            task_manager.do_send(TaskManagerReq::AddAppInstance(
                key.clone(),
                instance_key.clone(),
            ));
        }
        if let Some(app_info) = self.app_map.get_mut(&key) {
            if let Some(instance) = app_info.instance_map.get_mut(&instance_key) {
                instance.last_modified_time = last_modified_time;
                instance.enable = true;
                instance.healthy = true;
            } else {
                log::info!(
                    "register_app_instance|add instance:{:?},{},{}",
                    &key,
                    &instance_key,
                    last_modified_time
                );
                app_info.instance_map.insert(
                    instance_key.clone(),
                    AppInstance::new_with_time(instance_key, last_modified_time),
                );
            }
        } else {
            self.app_index
                .insert(key.namespace.clone(), key.app_name.clone());
            let mut app_info = AppInfo::new(
                key.app_name.clone(),
                key.namespace.clone(),
                EMPTY_ARC_STR.clone(),
                RegisterType::Auto,
                true,
            );
            log::info!(
                "register_app_instance|add instance:{:?},{},{}",
                &key,
                &instance_key,
                last_modified_time
            );
            app_info.instance_map.insert(
                instance_key.clone(),
                AppInstance::new_with_time(instance_key, last_modified_time),
            );
            self.app_map.insert(key, app_info);
        }
    }

    fn check_instance_timeout(&mut self) {
        let mut remove_list = vec![];
        let timeout_time = now_second_u32() - self.instance_timeout;
        for item in self.app_instance_timeout.timeout(timeout_time as u64) {
            let app_key = item.app_instance_key.build_app_key();
            if let Some(app_info) = self.app_map.get_mut(&app_key) {
                let mut can_remove = false;
                if let Some(instance) = app_info.instance_map.get(&item.app_instance_key.addr) {
                    can_remove = instance.last_modified_time <= timeout_time;
                }
                if can_remove {
                    log::info!(
                        "check_instance_timeout|remove instance:{:?},{}",
                        &app_key,
                        &item.app_instance_key.addr
                    );
                    app_info.instance_map.remove(&item.app_instance_key.addr);
                    remove_list.push(item.app_instance_key);
                }
            }
        }
        if let Some(task_manager) = self.task_manager.as_ref() {
            if !remove_list.is_empty() {
                task_manager.do_send(TaskManagerReq::RemoveAppInstances(remove_list));
            }
        }
    }

    fn heartbeat(&mut self, ctx: &mut Context<Self>) {
        ctx.run_later(std::time::Duration::from_secs(10), move |act, ctx| {
            act.check_instance_timeout();
            act.heartbeat(ctx);
        });
    }

    fn unregister_app_instance(&mut self, key: AppKey, instance_key: Arc<String>) {
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

    fn get_all_instances(&self) -> Vec<AppInstanceDto> {
        let mut list = Vec::new();
        for (key, app_info) in &self.app_map {
            for (addr, instance) in &app_info.instance_map {
                list.push(AppInstanceDto {
                    app_key: key.clone(),
                    instance_addr: addr.clone(),
                    last_modified_time: instance.last_modified_time,
                });
            }
        }
        list
    }

    fn app_route_request(&mut self, req: AppRouteRequest) -> anyhow::Result<AppRouteResponse> {
        match req {
            AppRouteRequest::RegisterInstance(param) => {
                self.register_app_instance(
                    param.app_key,
                    param.instance_addr,
                    param.last_modified_time,
                );
            }
            AppRouteRequest::UnregisterInstance(param) => {
                self.unregister_app_instance(param.app_key, param.instance_addr);
            }
            AppRouteRequest::GetAllInstanceAddrs => {
                let list = self.get_all_instances();
                return Ok(AppRouteResponse::AllInstanceAddrs(list));
            }
        }
        Ok(AppRouteResponse::None)
    }

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        for (key, app_info) in &self.app_map {
            if app_info.tmp {
                continue;
            }
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf);
                let value_do = app_info.to_do();
                writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: JOB_TABLE_NAME.clone(),
                key: key.build_key().as_bytes().to_vec(),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        Ok(())
    }

    fn load_snapshot_record(&mut self, record: SnapshotRecordDto) -> anyhow::Result<()> {
        let mut reader = BytesReader::from_bytes(&record.value);
        let value_do: AppInfoDo = reader.read_message(&record.value)?;
        let value: AppInfo = value_do.into();
        self.update_app_info(value);
        Ok(())
    }

    /// 数据加载完成后，维护应用实例生效、超时信息
    fn load_completed(&mut self, _ctx: &mut Context<Self>) -> anyhow::Result<()> {
        let now = now_second_u32();
        let start_time = now - self.instance_timeout;
        let mut add_keys = vec![];
        for (app_key, app_info) in self.app_map.iter() {
            for (instance_key, instance) in app_info.instance_map.iter() {
                let timeout = instance.last_modified_time;
                self.app_instance_timeout.add(
                    timeout as u64,
                    InstanceTimeoutInfo::new(
                        AppInstanceKey::new_with_app_key(app_key.clone(), instance_key.clone()),
                        timeout,
                    ),
                );
                if timeout > start_time {
                    log::info!("load_completed|add instance:{}", instance_key);
                    add_keys.push(AppInstanceKey::new_with_app_key(
                        app_key.clone(),
                        instance_key.clone(),
                    ));
                }
            }
        }
        self.check_instance_timeout();
        if let Some(task_manager) = self.task_manager.as_ref() {
            if !add_keys.is_empty() {
                task_manager.do_send(TaskManagerReq::AddAppInstances(add_keys));
            }
        }
        Ok(())
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
                log::warn!("AppManager|RegisterAppInstance, {:?},addr:{}", key, addr);
                //self.register_app_instance(key, addr, now_second_u32());
            }
            AppManagerReq::GetApp(key) => {
                return Ok(AppManagerResult::AppInfo(self.get_app_info(&key, true)));
            }
            AppManagerReq::UnregisterAppInstance(key, addr) => {
                self.unregister_app_instance(key, addr);
            }
            AppManagerReq::GetAppInstanceAddrs(key) => {
                let addrs = self.get_app_instance_addrs(key);
                return Ok(AppManagerResult::AppInstanceAddrs(addrs));
            }
            AppManagerReq::QueryApp(param) => {
                let (size, list) = self.query_page_info(&param);
                return Ok(AppManagerResult::AppPageInfo(size, list));
            }
            AppManagerReq::GetAllInstanceAddrs => {
                let list = self.get_all_instances();
                return Ok(AppManagerResult::AllInstanceAddrs(list));
            }
            AppManagerReq::AppRouteRequest(req) => {
                return Ok(AppManagerResult::AppRouteResponse(
                    self.app_route_request(req)?,
                ));
            }
        }
        Ok(AppManagerResult::None)
    }
}

impl Handler<AppManagerRaftReq> for AppManager {
    type Result = anyhow::Result<AppManagerRaftResult>;

    fn handle(&mut self, msg: AppManagerRaftReq, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            AppManagerRaftReq::UpdateApp(param) => {
                self.update_app(param);
            }
            AppManagerRaftReq::RemoveApp(key) => {
                self.app_map.remove(&key);
            }
            AppManagerRaftReq::RegisterInstance(param) => {
                self.register_app_instance(
                    param.app_key,
                    param.instance_addr,
                    param.last_modified_time,
                );
            }
            AppManagerRaftReq::UnregisterInstance(param) => {
                self.unregister_app_instance(param.app_key, param.instance_addr);
            }
        }
        Ok(AppManagerRaftResult::None)
    }
}

impl Handler<RaftApplyDataRequest> for AppManager {
    type Result = anyhow::Result<RaftApplyDataResponse>;

    fn handle(&mut self, msg: RaftApplyDataRequest, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            RaftApplyDataRequest::BuildSnapshot(writer) => {
                self.build_snapshot(writer)?;
            }
            RaftApplyDataRequest::LoadSnapshotRecord(record) => {
                self.load_snapshot_record(record)?;
            }
            RaftApplyDataRequest::LoadCompleted => {
                self.load_completed(ctx)?;
            }
        }
        Ok(RaftApplyDataResponse::None)
    }
}
