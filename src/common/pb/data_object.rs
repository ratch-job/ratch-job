// Automatically generated rust module for 'data_object.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use std::collections::HashMap;
type KVMap<K, V> = HashMap<K, V>;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct JobDo<'a> {
    pub id: u64,
    pub enable: bool,
    pub app_name: Cow<'a, str>,
    pub namespace: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub schedule_type: Cow<'a, str>,
    pub cron_value: Cow<'a, str>,
    pub delay_second: u32,
    pub interval_second: u32,
    pub run_mode: Cow<'a, str>,
    pub handle_name: Cow<'a, str>,
    pub trigger_param: Cow<'a, str>,
    pub router_strategy: Cow<'a, str>,
    pub past_due_strategy: Cow<'a, str>,
    pub blocking_strategy: Cow<'a, str>,
    pub timeout_second: u32,
    pub try_times: u32,
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub create_time: u64,
    pub retry_interval: u32,
}

impl<'a> MessageRead<'a> for JobDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_uint64(bytes)?,
                Ok(16) => msg.enable = r.read_bool(bytes)?,
                Ok(26) => msg.app_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(34) => msg.namespace = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(42) => msg.description = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(50) => msg.schedule_type = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(58) => msg.cron_value = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(64) => msg.delay_second = r.read_uint32(bytes)?,
                Ok(72) => msg.interval_second = r.read_uint32(bytes)?,
                Ok(82) => msg.run_mode = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(90) => msg.handle_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(98) => msg.trigger_param = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(106) => msg.router_strategy = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(114) => msg.past_due_strategy = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(122) => msg.blocking_strategy = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(128) => msg.timeout_second = r.read_uint32(bytes)?,
                Ok(136) => msg.try_times = r.read_uint32(bytes)?,
                Ok(144) => msg.version_id = r.read_uint64(bytes)?,
                Ok(152) => msg.last_modified_millis = r.read_uint64(bytes)?,
                Ok(160) => msg.create_time = r.read_uint64(bytes)?,
                Ok(168) => msg.retry_interval = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for JobDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.id == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.id) as u64) }
        + if self.enable == false { 0 } else { 1 + sizeof_varint(*(&self.enable) as u64) }
        + if self.app_name == "" { 0 } else { 1 + sizeof_len((&self.app_name).len()) }
        + if self.namespace == "" { 0 } else { 1 + sizeof_len((&self.namespace).len()) }
        + if self.description == "" { 0 } else { 1 + sizeof_len((&self.description).len()) }
        + if self.schedule_type == "" { 0 } else { 1 + sizeof_len((&self.schedule_type).len()) }
        + if self.cron_value == "" { 0 } else { 1 + sizeof_len((&self.cron_value).len()) }
        + if self.delay_second == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.delay_second) as u64) }
        + if self.interval_second == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.interval_second) as u64) }
        + if self.run_mode == "" { 0 } else { 1 + sizeof_len((&self.run_mode).len()) }
        + if self.handle_name == "" { 0 } else { 1 + sizeof_len((&self.handle_name).len()) }
        + if self.trigger_param == "" { 0 } else { 1 + sizeof_len((&self.trigger_param).len()) }
        + if self.router_strategy == "" { 0 } else { 1 + sizeof_len((&self.router_strategy).len()) }
        + if self.past_due_strategy == "" { 0 } else { 1 + sizeof_len((&self.past_due_strategy).len()) }
        + if self.blocking_strategy == "" { 0 } else { 1 + sizeof_len((&self.blocking_strategy).len()) }
        + if self.timeout_second == 0u32 { 0 } else { 2 + sizeof_varint(*(&self.timeout_second) as u64) }
        + if self.try_times == 0u32 { 0 } else { 2 + sizeof_varint(*(&self.try_times) as u64) }
        + if self.version_id == 0u64 { 0 } else { 2 + sizeof_varint(*(&self.version_id) as u64) }
        + if self.last_modified_millis == 0u64 { 0 } else { 2 + sizeof_varint(*(&self.last_modified_millis) as u64) }
        + if self.create_time == 0u64 { 0 } else { 2 + sizeof_varint(*(&self.create_time) as u64) }
        + if self.retry_interval == 0u32 { 0 } else { 2 + sizeof_varint(*(&self.retry_interval) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.id))?; }
        if self.enable != false { w.write_with_tag(16, |w| w.write_bool(*&self.enable))?; }
        if self.app_name != "" { w.write_with_tag(26, |w| w.write_string(&**&self.app_name))?; }
        if self.namespace != "" { w.write_with_tag(34, |w| w.write_string(&**&self.namespace))?; }
        if self.description != "" { w.write_with_tag(42, |w| w.write_string(&**&self.description))?; }
        if self.schedule_type != "" { w.write_with_tag(50, |w| w.write_string(&**&self.schedule_type))?; }
        if self.cron_value != "" { w.write_with_tag(58, |w| w.write_string(&**&self.cron_value))?; }
        if self.delay_second != 0u32 { w.write_with_tag(64, |w| w.write_uint32(*&self.delay_second))?; }
        if self.interval_second != 0u32 { w.write_with_tag(72, |w| w.write_uint32(*&self.interval_second))?; }
        if self.run_mode != "" { w.write_with_tag(82, |w| w.write_string(&**&self.run_mode))?; }
        if self.handle_name != "" { w.write_with_tag(90, |w| w.write_string(&**&self.handle_name))?; }
        if self.trigger_param != "" { w.write_with_tag(98, |w| w.write_string(&**&self.trigger_param))?; }
        if self.router_strategy != "" { w.write_with_tag(106, |w| w.write_string(&**&self.router_strategy))?; }
        if self.past_due_strategy != "" { w.write_with_tag(114, |w| w.write_string(&**&self.past_due_strategy))?; }
        if self.blocking_strategy != "" { w.write_with_tag(122, |w| w.write_string(&**&self.blocking_strategy))?; }
        if self.timeout_second != 0u32 { w.write_with_tag(128, |w| w.write_uint32(*&self.timeout_second))?; }
        if self.try_times != 0u32 { w.write_with_tag(136, |w| w.write_uint32(*&self.try_times))?; }
        if self.version_id != 0u64 { w.write_with_tag(144, |w| w.write_uint64(*&self.version_id))?; }
        if self.last_modified_millis != 0u64 { w.write_with_tag(152, |w| w.write_uint64(*&self.last_modified_millis))?; }
        if self.create_time != 0u64 { w.write_with_tag(160, |w| w.write_uint64(*&self.create_time))?; }
        if self.retry_interval != 0u32 { w.write_with_tag(168, |w| w.write_uint32(*&self.retry_interval))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct TaskTryLogDo<'a> {
    pub execution_time: u32,
    pub addr: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for TaskTryLogDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.execution_time = r.read_uint32(bytes)?,
                Ok(18) => msg.addr = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TaskTryLogDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.execution_time == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.execution_time) as u64) }
        + if self.addr == "" { 0 } else { 1 + sizeof_len((&self.addr).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.execution_time != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.execution_time))?; }
        if self.addr != "" { w.write_with_tag(18, |w| w.write_string(&**&self.addr))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct JobTaskDo<'a> {
    pub task_id: u64,
    pub job_id: u64,
    pub trigger_time: u32,
    pub instance_addr: Cow<'a, str>,
    pub trigger_message: Cow<'a, str>,
    pub status: Cow<'a, str>,
    pub finish_time: u32,
    pub callback_message: Cow<'a, str>,
    pub execution_time: u32,
    pub trigger_from: Cow<'a, str>,
    pub try_times: u32,
    pub try_logs: Vec<data_object::TaskTryLogDo<'a>>,
    pub retry_interval: u32,
    pub retry_count: u32,
    pub timeout_second: u32,
}

impl<'a> MessageRead<'a> for JobTaskDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.task_id = r.read_uint64(bytes)?,
                Ok(16) => msg.job_id = r.read_uint64(bytes)?,
                Ok(24) => msg.trigger_time = r.read_uint32(bytes)?,
                Ok(34) => msg.instance_addr = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(42) => msg.trigger_message = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(50) => msg.status = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(56) => msg.finish_time = r.read_uint32(bytes)?,
                Ok(66) => msg.callback_message = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(72) => msg.execution_time = r.read_uint32(bytes)?,
                Ok(82) => msg.trigger_from = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(88) => msg.try_times = r.read_uint32(bytes)?,
                Ok(98) => msg.try_logs.push(r.read_message::<data_object::TaskTryLogDo>(bytes)?),
                Ok(104) => msg.retry_interval = r.read_uint32(bytes)?,
                Ok(112) => msg.retry_count = r.read_uint32(bytes)?,
                Ok(120) => msg.timeout_second = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for JobTaskDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.task_id == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.task_id) as u64) }
        + if self.job_id == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.job_id) as u64) }
        + if self.trigger_time == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.trigger_time) as u64) }
        + if self.instance_addr == "" { 0 } else { 1 + sizeof_len((&self.instance_addr).len()) }
        + if self.trigger_message == "" { 0 } else { 1 + sizeof_len((&self.trigger_message).len()) }
        + if self.status == "" { 0 } else { 1 + sizeof_len((&self.status).len()) }
        + if self.finish_time == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.finish_time) as u64) }
        + if self.callback_message == "" { 0 } else { 1 + sizeof_len((&self.callback_message).len()) }
        + if self.execution_time == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.execution_time) as u64) }
        + if self.trigger_from == "" { 0 } else { 1 + sizeof_len((&self.trigger_from).len()) }
        + if self.try_times == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.try_times) as u64) }
        + self.try_logs.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + if self.retry_interval == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.retry_interval) as u64) }
        + if self.retry_count == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.retry_count) as u64) }
        + if self.timeout_second == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.timeout_second) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.task_id != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.task_id))?; }
        if self.job_id != 0u64 { w.write_with_tag(16, |w| w.write_uint64(*&self.job_id))?; }
        if self.trigger_time != 0u32 { w.write_with_tag(24, |w| w.write_uint32(*&self.trigger_time))?; }
        if self.instance_addr != "" { w.write_with_tag(34, |w| w.write_string(&**&self.instance_addr))?; }
        if self.trigger_message != "" { w.write_with_tag(42, |w| w.write_string(&**&self.trigger_message))?; }
        if self.status != "" { w.write_with_tag(50, |w| w.write_string(&**&self.status))?; }
        if self.finish_time != 0u32 { w.write_with_tag(56, |w| w.write_uint32(*&self.finish_time))?; }
        if self.callback_message != "" { w.write_with_tag(66, |w| w.write_string(&**&self.callback_message))?; }
        if self.execution_time != 0u32 { w.write_with_tag(72, |w| w.write_uint32(*&self.execution_time))?; }
        if self.trigger_from != "" { w.write_with_tag(82, |w| w.write_string(&**&self.trigger_from))?; }
        if self.try_times != 0u32 { w.write_with_tag(88, |w| w.write_uint32(*&self.try_times))?; }
        for s in &self.try_logs { w.write_with_tag(98, |w| w.write_message(s))?; }
        if self.retry_interval != 0u32 { w.write_with_tag(104, |w| w.write_uint32(*&self.retry_interval))?; }
        if self.retry_count != 0u32 { w.write_with_tag(112, |w| w.write_uint32(*&self.retry_count))?; }
        if self.timeout_second != 0u32 { w.write_with_tag(120, |w| w.write_uint32(*&self.timeout_second))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct AppInstanceDo<'a> {
    pub addr: Cow<'a, str>,
    pub last_modified_time: u32,
    pub token: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for AppInstanceDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.addr = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.last_modified_time = r.read_uint32(bytes)?,
                Ok(26) => msg.token = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for AppInstanceDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.addr == "" { 0 } else { 1 + sizeof_len((&self.addr).len()) }
        + if self.last_modified_time == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.last_modified_time) as u64) }
        + if self.token == "" { 0 } else { 1 + sizeof_len((&self.token).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.addr != "" { w.write_with_tag(10, |w| w.write_string(&**&self.addr))?; }
        if self.last_modified_time != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.last_modified_time))?; }
        if self.token != "" { w.write_with_tag(26, |w| w.write_string(&**&self.token))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct AppInfoDo<'a> {
    pub app_name: Cow<'a, str>,
    pub namespace: Cow<'a, str>,
    pub label: Cow<'a, str>,
    pub register_type: Cow<'a, str>,
    pub tmp: bool,
    pub instances: Vec<data_object::AppInstanceDo<'a>>,
}

impl<'a> MessageRead<'a> for AppInfoDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.app_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.namespace = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(26) => msg.label = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(34) => msg.register_type = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(40) => msg.tmp = r.read_bool(bytes)?,
                Ok(50) => msg.instances.push(r.read_message::<data_object::AppInstanceDo>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for AppInfoDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.app_name == "" { 0 } else { 1 + sizeof_len((&self.app_name).len()) }
        + if self.namespace == "" { 0 } else { 1 + sizeof_len((&self.namespace).len()) }
        + if self.label == "" { 0 } else { 1 + sizeof_len((&self.label).len()) }
        + if self.register_type == "" { 0 } else { 1 + sizeof_len((&self.register_type).len()) }
        + if self.tmp == false { 0 } else { 1 + sizeof_varint(*(&self.tmp) as u64) }
        + self.instances.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.app_name != "" { w.write_with_tag(10, |w| w.write_string(&**&self.app_name))?; }
        if self.namespace != "" { w.write_with_tag(18, |w| w.write_string(&**&self.namespace))?; }
        if self.label != "" { w.write_with_tag(26, |w| w.write_string(&**&self.label))?; }
        if self.register_type != "" { w.write_with_tag(34, |w| w.write_string(&**&self.register_type))?; }
        if self.tmp != false { w.write_with_tag(40, |w| w.write_bool(*&self.tmp))?; }
        for s in &self.instances { w.write_with_tag(50, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct CacheItemDo<'a> {
    pub cache_type: u32,
    pub key: Cow<'a, str>,
    pub data: Cow<'a, [u8]>,
    pub timeout: i32,
}

impl<'a> MessageRead<'a> for CacheItemDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.cache_type = r.read_uint32(bytes)?,
                Ok(18) => msg.key = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(26) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(32) => msg.timeout = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for CacheItemDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.cache_type == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.cache_type) as u64) }
        + if self.key == "" { 0 } else { 1 + sizeof_len((&self.key).len()) }
        + if self.data == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.data).len()) }
        + if self.timeout == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.timeout) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.cache_type != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.cache_type))?; }
        if self.key != "" { w.write_with_tag(18, |w| w.write_string(&**&self.key))?; }
        if self.data != Cow::Borrowed(b"") { w.write_with_tag(26, |w| w.write_bytes(&**&self.data))?; }
        if self.timeout != 0i32 { w.write_with_tag(32, |w| w.write_int32(*&self.timeout))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct PrivilegeGroupDo<'a> {
    pub enabled: bool,
    pub whitelist_is_all: bool,
    pub whitelist: Vec<Cow<'a, str>>,
    pub blacklist_is_all: bool,
    pub blacklist: Vec<Cow<'a, str>>,
}

impl<'a> MessageRead<'a> for PrivilegeGroupDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.enabled = r.read_bool(bytes)?,
                Ok(16) => msg.whitelist_is_all = r.read_bool(bytes)?,
                Ok(26) => msg.whitelist.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(32) => msg.blacklist_is_all = r.read_bool(bytes)?,
                Ok(42) => msg.blacklist.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PrivilegeGroupDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.enabled == false { 0 } else { 1 + sizeof_varint(*(&self.enabled) as u64) }
        + if self.whitelist_is_all == false { 0 } else { 1 + sizeof_varint(*(&self.whitelist_is_all) as u64) }
        + self.whitelist.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + if self.blacklist_is_all == false { 0 } else { 1 + sizeof_varint(*(&self.blacklist_is_all) as u64) }
        + self.blacklist.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.enabled != false { w.write_with_tag(8, |w| w.write_bool(*&self.enabled))?; }
        if self.whitelist_is_all != false { w.write_with_tag(16, |w| w.write_bool(*&self.whitelist_is_all))?; }
        for s in &self.whitelist { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        if self.blacklist_is_all != false { w.write_with_tag(32, |w| w.write_bool(*&self.blacklist_is_all))?; }
        for s in &self.blacklist { w.write_with_tag(42, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct UserInfoDo<'a> {
    pub username: Cow<'a, str>,
    pub nickname: Cow<'a, str>,
    pub password_hash: Cow<'a, str>,
    pub gmt_create: i64,
    pub gmt_modified: i64,
    pub enable: bool,
    pub roles: Vec<Cow<'a, str>>,
    pub extend_info: KVMap<Cow<'a, str>, Cow<'a, str>>,
    pub namespace_privilege: Option<data_object::PrivilegeGroupDo<'a>>,
    pub app_privilege: Option<data_object::PrivilegeGroupDo<'a>>,
}

impl<'a> MessageRead<'a> for UserInfoDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.username = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.nickname = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(34) => msg.password_hash = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(40) => msg.gmt_create = r.read_int64(bytes)?,
                Ok(48) => msg.gmt_modified = r.read_int64(bytes)?,
                Ok(56) => msg.enable = r.read_bool(bytes)?,
                Ok(66) => msg.roles.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(74) => {
                    let (key, value) = r.read_map(bytes, |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?), |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?))?;
                    msg.extend_info.insert(key, value);
                }
                Ok(82) => msg.namespace_privilege = Some(r.read_message::<data_object::PrivilegeGroupDo>(bytes)?),
                Ok(90) => msg.app_privilege = Some(r.read_message::<data_object::PrivilegeGroupDo>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for UserInfoDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.username == "" { 0 } else { 1 + sizeof_len((&self.username).len()) }
        + if self.nickname == "" { 0 } else { 1 + sizeof_len((&self.nickname).len()) }
        + if self.password_hash == "" { 0 } else { 1 + sizeof_len((&self.password_hash).len()) }
        + if self.gmt_create == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.gmt_create) as u64) }
        + if self.gmt_modified == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.gmt_modified) as u64) }
        + if self.enable == false { 0 } else { 1 + sizeof_varint(*(&self.enable) as u64) }
        + self.roles.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + self.extend_info.iter().map(|(k, v)| 1 + sizeof_len(2 + sizeof_len((k).len()) + sizeof_len((v).len()))).sum::<usize>()
        + self.namespace_privilege.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.app_privilege.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.username != "" { w.write_with_tag(10, |w| w.write_string(&**&self.username))?; }
        if self.nickname != "" { w.write_with_tag(18, |w| w.write_string(&**&self.nickname))?; }
        if self.password_hash != "" { w.write_with_tag(34, |w| w.write_string(&**&self.password_hash))?; }
        if self.gmt_create != 0i64 { w.write_with_tag(40, |w| w.write_int64(*&self.gmt_create))?; }
        if self.gmt_modified != 0i64 { w.write_with_tag(48, |w| w.write_int64(*&self.gmt_modified))?; }
        if self.enable != false { w.write_with_tag(56, |w| w.write_bool(*&self.enable))?; }
        for s in &self.roles { w.write_with_tag(66, |w| w.write_string(&**s))?; }
        for (k, v) in self.extend_info.iter() { w.write_with_tag(74, |w| w.write_map(2 + sizeof_len((k).len()) + sizeof_len((v).len()), 10, |w| w.write_string(&**k), 18, |w| w.write_string(&**v)))?; }
        if let Some(ref s) = self.namespace_privilege { w.write_with_tag(82, |w| w.write_message(s))?; }
        if let Some(ref s) = self.app_privilege { w.write_with_tag(90, |w| w.write_message(s))?; }
        Ok(())
    }
}

