use crate::common::constant::DEFAULT_XXL_NAMESPACE;
use crate::common::pb::data_object::NamespaceDo;
use crate::job::core::JobManager;
use crate::namespace::model::actor_model::{
    NamespaceManagerRaftReq, NamespaceManagerRaftResult, NamespaceManagerReq,
    NamespaceManagerResult,
};
use crate::namespace::model::namespace::{
    Namespace, NamespaceInfo, NamespaceParam, NamespaceQueryParam, NamespaceWrap,
};
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::{RaftApplyDataRequest, RaftApplyDataResponse};
use crate::raft::store::raftsnapshot::{SnapshotWriterActor, SnapshotWriterRequest};
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use quick_protobuf::{BytesReader, Writer};
use std::collections::BTreeMap;
use std::sync::Arc;

#[bean(inject)]
pub struct NamespaceManager {
    namespace_map: BTreeMap<Arc<String>, NamespaceWrap>,
    data_load_completed: bool,
    job_manager: Option<Addr<JobManager>>,
    version: u64,
}

impl NamespaceManager {
    pub fn new() -> Self {
        let mut namespace_map = BTreeMap::new();
        namespace_map.insert(
            DEFAULT_XXL_NAMESPACE.clone(),
            NamespaceWrap::new(
                Arc::new(Namespace {
                    id: DEFAULT_XXL_NAMESPACE.clone(),
                    name: DEFAULT_XXL_NAMESPACE.to_string(),
                    r#type: "0".to_string(),
                }),
                0,
            ),
        );
        NamespaceManager {
            namespace_map,
            data_load_completed: false,
            job_manager: None,
            version: 1,
        }
    }

    fn update_namespace(
        &mut self,
        param: NamespaceParam,
        from_weak: bool,
    ) -> anyhow::Result<Arc<NamespaceInfo>> {
        let id = match param.id {
            Some(ref id) if !id.is_empty() => id.clone(),
            _ => return Err(anyhow::anyhow!("namespace id cannot be empty")),
        };
        if id.as_str() == DEFAULT_XXL_NAMESPACE.as_str() {
            return Err(anyhow::anyhow!("jump default namespace"));
        }
        if param.name.is_empty() {
            log::error!("Failed to update namespace: name is empty, id={}", id);
            return Err(anyhow::anyhow!("namespace name cannot be empty"));
        }
        let from_type = if from_weak {
            "weak".to_string()
        } else {
            "0".to_string()
        };
        if let Some(wrap) = self.namespace_map.get_mut(&id) {
            let ns = Arc::make_mut(&mut wrap.namespace);
            ns.name = param.name;
            ns.r#type = from_type.clone();
            let info = NamespaceInfo::from_namespace(&wrap.namespace);
            log::info!(
                "Updated namespace: id={}, name={}, type={}",
                id,
                info.name,
                info.r#type
            );
            return Ok(Arc::new(info));
        }
        let namespace = Namespace {
            id: id.clone(),
            name: param.name,
            r#type: from_type,
        };
        let wrap = NamespaceWrap::new(Arc::new(namespace), self.version);
        self.version += 1;
        let info = NamespaceInfo::from_namespace(&wrap.namespace);
        self.namespace_map.insert(id.clone(), wrap);
        log::info!(
            "Created namespace: id={}, name={}, type={}",
            id,
            info.name,
            info.r#type
        );
        Ok(Arc::new(info))
    }

    fn remove_namespace(&mut self, id: Arc<String>) -> anyhow::Result<()> {
        if id.as_str() == DEFAULT_XXL_NAMESPACE.as_str() {
            return Ok(());
        }
        self.namespace_map.remove(&id).ok_or_else(|| {
            log::error!("Failed to remove namespace: namespace not found, id={}", id);
            anyhow::anyhow!("namespace not found")
        })?;
        log::info!("Removed namespace: id={}", id);
        Ok(())
    }

    fn get_namespace(&self, id: &Arc<String>) -> Option<Arc<NamespaceInfo>> {
        self.namespace_map
            .get(id)
            .map(|w| Arc::new(NamespaceInfo::from_namespace(&w.namespace)))
    }

    fn query_namespace_page(&self, param: NamespaceQueryParam) -> (usize, Vec<NamespaceInfo>) {
        let mut list: Vec<&NamespaceWrap> = self.namespace_map.values().collect();
        if let Some(ref ns_type) = param.r#type {
            list.retain(|w| &w.namespace.r#type == ns_type);
        }
        let total = list.len();
        let page = param.page.unwrap_or(1).max(1) as usize;
        let page_size = param.page_size.unwrap_or(10).max(1) as usize;
        let offset = (page - 1) * page_size;
        let paged: Vec<NamespaceInfo> = list
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(|w| NamespaceInfo::from_namespace(&w.namespace))
            .collect();
        (total, paged)
    }

    fn query_list(&self) -> Vec<NamespaceInfo> {
        self.namespace_map
            .values()
            .map(|w| NamespaceInfo::from_namespace(&w.namespace))
            .collect()
    }
    fn set_weak(&mut self, id: Arc<String>) {
        if let Some(_ns) = self.namespace_map.get(&id) {
            return;
        }
        let param = NamespaceParam {
            id: Some(id.clone()),
            name: id.to_string(),
        };
        self.update_namespace(param, true).ok();
    }

    fn remove_weak(&mut self, id: Arc<String>) {
        if let Some(ns) = self.namespace_map.get(&id) {
            if ns.namespace.r#type.as_str() == "weak" {
                self.namespace_map.remove(&id);
            }
        }
    }

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        for (key, wrap) in self.namespace_map.iter() {
            if wrap.namespace.id.as_str() == DEFAULT_XXL_NAMESPACE.as_str() {
                continue;
            }
            if wrap.namespace.r#type.as_str() == "weak" {
                continue;
            }
            let mut buf = Vec::new();
            {
                let mut pb_writer = Writer::new(&mut buf);
                let value_do = wrap.namespace.to_do();
                pb_writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: Arc::new("T_NAMESPACE".to_string()),
                key: key.to_string().into_bytes(),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        Ok(())
    }

    fn load_snapshot_record(&mut self, record: SnapshotRecordDto) -> anyhow::Result<()> {
        let mut reader = BytesReader::from_bytes(&record.value);
        let ns_do: NamespaceDo = reader.read_message(&record.value)?;
        let namespace: Namespace = ns_do.into();
        let wrap = NamespaceWrap::new(Arc::new(namespace.clone()), self.version);
        self.version += 1;
        self.namespace_map.insert(namespace.id, wrap);
        Ok(())
    }

    fn load_completed(&mut self, _ctx: &mut Context<Self>) -> anyhow::Result<()> {
        self.data_load_completed = true;
        log::info!("NamespaceManager load completed");
        Ok(())
    }
}

impl Actor for NamespaceManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("NamespaceManager started");
    }
}

impl Inject for NamespaceManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.job_manager = factory_data.get_actor();
    }
}

impl Handler<NamespaceManagerRaftReq> for NamespaceManager {
    type Result = anyhow::Result<NamespaceManagerRaftResult>;

    fn handle(&mut self, msg: NamespaceManagerRaftReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            NamespaceManagerRaftReq::UpdateNamespace(param) => {
                self.update_namespace(param, false).ok();
                Ok(NamespaceManagerRaftResult::None)
            }
            NamespaceManagerRaftReq::Remove(id) => {
                self.remove_namespace(id)?;
                Ok(NamespaceManagerRaftResult::None)
            }
        }
    }
}

impl Handler<NamespaceManagerReq> for NamespaceManager {
    type Result = anyhow::Result<NamespaceManagerResult>;

    fn handle(&mut self, msg: NamespaceManagerReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            NamespaceManagerReq::GetNamespace(id) => {
                let info = self.get_namespace(&id);
                Ok(NamespaceManagerResult::NamespaceInfo(info))
            }
            NamespaceManagerReq::SetWeak(id) => {
                self.set_weak(id);
                Ok(NamespaceManagerResult::None)
            }
            NamespaceManagerReq::RemoveWeak(id) => {
                self.remove_weak(id);
                Ok(NamespaceManagerResult::None)
            }
            NamespaceManagerReq::QueryNamespace(param) => {
                let (total, list) = self.query_namespace_page(param);
                Ok(NamespaceManagerResult::NamespacePageInfo(total, list))
            }
            NamespaceManagerReq::QueryList => {
                let list = self.query_list();
                Ok(NamespaceManagerResult::NamespaceList(list))
            }
        }
    }
}

impl Handler<RaftApplyDataRequest> for NamespaceManager {
    type Result = anyhow::Result<RaftApplyDataResponse>;

    fn handle(&mut self, msg: RaftApplyDataRequest, ctx: &mut Self::Context) -> Self::Result {
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
