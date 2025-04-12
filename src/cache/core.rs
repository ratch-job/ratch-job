use crate::app::model::AppInfo;
use crate::cache::actor_model::{
    CacheManagerLocalReq, CacheManagerRaftReq, CacheManagerRaftResult, SetInfo,
};
use crate::cache::model::{CacheKey, CacheValue};
use crate::common::constant::CACHE_TABLE_NAME;
use crate::common::datetime_utils::now_second_i32;
use crate::common::pb::data_object::{AppInfoDo, CacheItemDo};
use crate::job::model::enum_type::PastDueStrategy::Default;
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::{RaftApplyDataRequest, RaftApplyDataResponse};
use crate::raft::store::raftsnapshot::{SnapshotWriterActor, SnapshotWriterRequest};
use actix::prelude::*;
use bean_factory::{bean, Inject};
use inner_mem_cache::{MemCache, TimeoutSet};
use quick_protobuf::{BytesReader, Writer};
use std::collections::HashMap;
use std::sync::Arc;

pub struct CacheItem {
    pub expire: i32,
    pub value: Arc<CacheValue>,
}

impl CacheItem {
    pub fn new(value: Arc<CacheValue>, expire: i32) -> Self {
        CacheItem { expire, value }
    }
}

#[bean(inject)]
#[derive(Default)]
pub struct CacheManager {
    cache: HashMap<CacheKey, CacheItem>,
    time_set: TimeoutSet<CacheKey>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            time_set: TimeoutSet::new(),
        }
    }

    fn clear_time_out(&mut self) {
        let now_second = now_second_i32();
        for key in self.time_set.timeout(now_second as u64) {
            if let Some(v) = self.cache.get(&key) {
                if v.expire > -1 && v.expire < now_second {
                    self.cache.remove(&key);
                }
            }
        }
    }

    fn heartbeat(&mut self, ctx: &mut Context<Self>) {
        ctx.run_later(std::time::Duration::from_secs(1), move |act, ctx| {
            act.clear_time_out();
            act.heartbeat(ctx);
        });
    }

    fn do_set_arc(&mut self, key: CacheKey, value: Arc<CacheValue>, expire: i32) {
        self.cache
            .insert(key.clone(), CacheItem::new(value, expire));
        if expire < 0 {
            //永久不过期
            return;
        }
        self.time_set.add(expire as u64, key);
    }
    fn do_set(&mut self, key: CacheKey, value: CacheValue, expire: i32) {
        self.do_set_arc(key, Arc::new(value), expire)
    }

    fn get_valid_value(&self, key: &CacheKey) -> Option<&Arc<CacheValue>> {
        if let Some(v) = self.cache.get(key) {
            if v.expire > -1 && v.expire < now_second_i32() {
                None
            } else {
                Some(&v.value)
            }
        } else {
            None
        }
    }

    fn set_nx(&mut self, key: CacheKey, value: CacheValue, expire: i32) -> CacheManagerRaftResult {
        if let Some(_v) = self.get_valid_value(&key) {
            CacheManagerRaftResult::Nil
        } else {
            self.do_set(key, value, expire);
            CacheManagerRaftResult::Ok
        }
    }

    fn set_xx(&mut self, key: CacheKey, value: CacheValue, expire: i32) -> CacheManagerRaftResult {
        if let Some(_v) = self.get_valid_value(&key) {
            self.do_set(key, value, expire);
            CacheManagerRaftResult::Ok
        } else {
            CacheManagerRaftResult::Nil
        }
    }

    fn set_value(
        &mut self,
        key: CacheKey,
        value: CacheValue,
        expire: i32,
    ) -> CacheManagerRaftResult {
        self.do_set(key, value, expire);
        CacheManagerRaftResult::Ok
    }

    fn set(&mut self, set_info: SetInfo) -> CacheManagerRaftResult {
        if set_info.nx {
            self.set_nx(set_info.key, set_info.value, set_info.ttl + set_info.now)
        } else if set_info.xx {
            self.set_xx(set_info.key, set_info.value, set_info.ttl + set_info.now)
        } else {
            self.set_value(set_info.key, set_info.value, set_info.ttl + set_info.now)
        }
    }

    fn get_set(&mut self, set_info: SetInfo) -> CacheManagerRaftResult {
        let r = if let Some(v) = self.get_valid_value(&set_info.key) {
            CacheManagerRaftResult::Value(v.clone())
        } else {
            CacheManagerRaftResult::None
        };
        self.do_set(set_info.key, set_info.value, set_info.ttl + set_info.now);
        r
    }

    fn get_value(&mut self, key: &CacheKey) -> CacheManagerRaftResult {
        if let Some(v) = self.get_valid_value(key) {
            CacheManagerRaftResult::Value(v.clone())
        } else {
            CacheManagerRaftResult::None
        }
    }

    fn remove(&mut self, key: &CacheKey) -> CacheManagerRaftResult {
        self.cache.remove(key);
        CacheManagerRaftResult::Ok
    }

    fn exists(&mut self, key: &CacheKey) -> CacheManagerRaftResult {
        if let Some(_v) = self.get_valid_value(key) {
            CacheManagerRaftResult::Exists(true)
        } else {
            CacheManagerRaftResult::Exists(false)
        }
    }

    fn expire(&mut self, key: CacheKey, expire: i32) -> CacheManagerRaftResult {
        if let Some(v) = self.get_valid_value(&key) {
            self.do_set_arc(key, v.clone(), expire);
            CacheManagerRaftResult::Ok
        } else {
            CacheManagerRaftResult::Nil
        }
    }

    fn get_ttl(&mut self, key: &CacheKey) -> CacheManagerRaftResult {
        let t = if let Some(v) = self.cache.get(key) {
            let t = v.expire - now_second_i32();
            if t > -2 {
                t
            } else {
                -2
            }
        } else {
            -2
        };
        CacheManagerRaftResult::Ttl(t)
    }

    fn incr(&mut self, key: CacheKey, expire: i32) -> CacheManagerRaftResult {
        if let Some(v) = self.get_valid_value(&key) {
            if let Some(v) = v.try_to_number() {
                if v == i64::MAX {
                    return CacheManagerRaftResult::Nil;
                }
                let value = Arc::new(CacheValue::Number(v + 1));
                self.do_set_arc(key, value.clone(), expire);
                CacheManagerRaftResult::Value(value)
            } else {
                CacheManagerRaftResult::Nil
            }
        } else {
            let value = Arc::new(CacheValue::Number(1));
            self.do_set_arc(key, value.clone(), expire);
            CacheManagerRaftResult::Value(value)
        }
    }

    fn decr(&mut self, key: CacheKey, expire: i32) -> CacheManagerRaftResult {
        if let Some(v) = self.get_valid_value(&key) {
            if let Some(v) = v.try_to_number() {
                if v == i64::MIN {
                    return CacheManagerRaftResult::Nil;
                }
                let value = Arc::new(CacheValue::Number(v - 1));
                self.do_set_arc(key, value.clone(), expire);
                CacheManagerRaftResult::Value(value)
            } else {
                CacheManagerRaftResult::Nil
            }
        } else {
            let value = Arc::new(CacheValue::Number(-1));
            self.do_set_arc(key, value.clone(), expire);
            CacheManagerRaftResult::Value(value)
        }
    }

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        let now = now_second_i32();
        for (key, v) in self.cache.iter() {
            if v.expire > -1 && v.expire < now {
                continue;
            }
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf);
                let value_do = v.value.to_do(key);
                writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: CACHE_TABLE_NAME.clone(),
                key: key.to_key_string().into_bytes(),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        Ok(())
    }

    fn load_snapshot_record(&mut self, record: SnapshotRecordDto) -> anyhow::Result<()> {
        let mut reader = BytesReader::from_bytes(&record.value);
        let value_do: CacheItemDo = reader.read_message(&record.value)?;
        let key = CacheKey::from_db_key(record.key)?;
        let value = CacheValue::from_bytes(&value_do.data, key.cache_type.clone())?;
        self.do_set(key, value, value_do.timeout);
        Ok(())
    }

    fn load_completed(&mut self, _ctx: &mut Context<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Actor for CacheManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("CacheManager actor started");
        self.heartbeat(ctx);
    }
}

impl Inject for CacheManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        _factory_data: bean_factory::FactoryData,
        _factory: bean_factory::BeanFactory,
        _ctx: &mut Self::Context,
    ) {
    }
}

impl Handler<CacheManagerLocalReq> for CacheManager {
    type Result = anyhow::Result<CacheManagerRaftResult>;

    fn handle(&mut self, req: CacheManagerLocalReq, _ctx: &mut Self::Context) -> Self::Result {
        match req {
            CacheManagerLocalReq::Get(key) => Ok(self.get_value(&key)),
            CacheManagerLocalReq::Exists(key) => Ok(self.exists(&key)),
            CacheManagerLocalReq::Ttl(key) => Ok(self.get_ttl(&key)),
        }
    }
}

impl Handler<CacheManagerRaftReq> for CacheManager {
    type Result = anyhow::Result<CacheManagerRaftResult>;

    fn handle(&mut self, req: CacheManagerRaftReq, _ctx: &mut Self::Context) -> Self::Result {
        match req {
            CacheManagerRaftReq::Set(set_info) => Ok(self.set(set_info)),
            CacheManagerRaftReq::GetSet(set_info) => Ok(self.get_set(set_info)),
            CacheManagerRaftReq::Get(key) => Ok(self.get_value(&key)),
            CacheManagerRaftReq::Remove(key) => Ok(self.remove(&key)),
            CacheManagerRaftReq::Exists(key) => Ok(self.exists(&key)),
            CacheManagerRaftReq::Expire(key, expire) => Ok(self.expire(key, expire)),
            CacheManagerRaftReq::Ttl(key) => Ok(self.get_ttl(&key)),
            CacheManagerRaftReq::Incr(key, expire) => Ok(self.incr(key, expire)),
            CacheManagerRaftReq::Decr(key, expire) => Ok(self.decr(key, expire)),
        }
    }
}

impl Handler<RaftApplyDataRequest> for CacheManager {
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
