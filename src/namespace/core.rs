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
    namespace_map: BTreeMap<String, NamespaceWrap>,
    data_load_completed: bool,
    job_manager: Option<Addr<JobManager>>,
}

impl NamespaceManager {
    pub fn new() -> Self {
        NamespaceManager {
            namespace_map: BTreeMap::new(),
            data_load_completed: false,
            job_manager: None,
        }
    }

    fn add_namespace(&mut self, param: NamespaceParam) -> anyhow::Result<Arc<NamespaceInfo>> {
        let id = match param.id {
            Some(ref id) if !id.is_empty() => id.clone(),
            _ => uuid::Uuid::new_v4().to_string(),
        };
        if param.name.is_empty() {
            log::error!("Failed to add namespace: name is empty, id={}", id);
            return Err(anyhow::anyhow!("namespace name cannot be empty"));
        }
        if param.r#type.is_empty() {
            log::error!("Failed to add namespace: type is empty, id={}", id);
            return Err(anyhow::anyhow!("namespace type cannot be empty"));
        }
        if let Some(wrap) = self.namespace_map.get_mut(&id) {
            let ns = Arc::make_mut(&mut wrap.namespace);
            ns.name = param.name;
            ns.r#type = param.r#type;
            wrap.name_lower = ns.name.to_lowercase();
            wrap.type_lower = ns.r#type.to_lowercase();
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
            r#type: param.r#type,
        };
        let wrap = NamespaceWrap::new(Arc::new(namespace));
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

    fn update_namespace(&mut self, param: NamespaceParam) -> anyhow::Result<Arc<NamespaceInfo>> {
        let id = match param.id {
            Some(ref id) if !id.is_empty() => id.clone(),
            _ => {
                log::error!("Failed to update namespace: id is empty");
                return Err(anyhow::anyhow!("namespace id cannot be empty"));
            }
        };
        let wrap = self.namespace_map.get_mut(&id).ok_or_else(|| {
            log::error!("Failed to update namespace: namespace not found, id={}", id);
            anyhow::anyhow!("namespace not found")
        })?;
        let ns = Arc::make_mut(&mut wrap.namespace);
        ns.name = param.name;
        ns.r#type = param.r#type;
        wrap.name_lower = ns.name.to_lowercase();
        wrap.type_lower = ns.r#type.to_lowercase();
        let info = NamespaceInfo::from_namespace(&wrap.namespace);
        log::info!(
            "Updated namespace: id={}, name={}, type={}",
            id,
            info.name,
            info.r#type
        );
        Ok(Arc::new(info))
    }

    fn remove_namespace(&mut self, id: String) -> anyhow::Result<()> {
        self.namespace_map.remove(&id).ok_or_else(|| {
            log::error!("Failed to remove namespace: namespace not found, id={}", id);
            anyhow::anyhow!("namespace not found")
        })?;
        log::info!("Removed namespace: id={}", id);
        Ok(())
    }

    fn get_namespace(&self, id: &str) -> Option<Arc<NamespaceInfo>> {
        self.namespace_map
            .get(id)
            .map(|w| Arc::new(NamespaceInfo::from_namespace(&w.namespace)))
    }

    fn query_namespace(&self, param: NamespaceQueryParam) -> (usize, Vec<NamespaceInfo>) {
        let mut list: Vec<&NamespaceWrap> = self.namespace_map.values().collect();
        if let Some(ref ns_type) = param.r#type {
            let type_lower = ns_type.to_lowercase();
            list.retain(|w| w.type_lower == type_lower);
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

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        for (key, wrap) in self.namespace_map.iter() {
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
        let wrap = NamespaceWrap::new(Arc::new(namespace.clone()));
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
            NamespaceManagerRaftReq::AddNamespace(param) => {
                let info = self.add_namespace(param)?;
                Ok(NamespaceManagerRaftResult::NamespaceInfo(info))
            }
            NamespaceManagerRaftReq::UpdateNamespace(param) => {
                let info = self.update_namespace(param)?;
                Ok(NamespaceManagerRaftResult::NamespaceInfo(info))
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
            NamespaceManagerReq::QueryNamespace(param) => {
                let (total, list) = self.query_namespace(param);
                Ok(NamespaceManagerResult::NamespacePageInfo(total, list))
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
