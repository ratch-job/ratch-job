// Automatically generated rust module for 'data_object.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
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
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.app_name != "" { w.write_with_tag(10, |w| w.write_string(&**&self.app_name))?; }
        if self.namespace != "" { w.write_with_tag(18, |w| w.write_string(&**&self.namespace))?; }
        if self.label != "" { w.write_with_tag(26, |w| w.write_string(&**&self.label))?; }
        if self.register_type != "" { w.write_with_tag(34, |w| w.write_string(&**&self.register_type))?; }
        if self.tmp != false { w.write_with_tag(40, |w| w.write_bool(*&self.tmp))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct NotifyConfigDo<'a> {
    pub id: u64,
    pub enable: bool,
    pub app_name: Cow<'a, str>,
    pub namespace: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub channel_type: Cow<'a, str>,
    pub channel_sub_type: Cow<'a, str>,
    pub url: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub username: Cow<'a, str>,
    pub password: Cow<'a, str>,
    pub token: Cow<'a, str>,
    pub version_id: u64,
    pub last_modified_millis: u64,
    pub create_time: u64,
}

impl<'a> MessageRead<'a> for NotifyConfigDo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_uint64(bytes)?,
                Ok(16) => msg.enable = r.read_bool(bytes)?,
                Ok(26) => msg.app_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(34) => msg.namespace = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(42) => msg.name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(50) => msg.channel_type = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(58) => msg.channel_sub_type = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(66) => msg.url = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(74) => msg.email = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(82) => msg.username = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(90) => msg.password = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(98) => msg.token = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(104) => msg.version_id = r.read_uint64(bytes)?,
                Ok(112) => msg.last_modified_millis = r.read_uint64(bytes)?,
                Ok(120) => msg.create_time = r.read_uint64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for NotifyConfigDo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.id == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.id) as u64) }
        + if self.enable == false { 0 } else { 1 + sizeof_varint(*(&self.enable) as u64) }
        + if self.app_name == "" { 0 } else { 1 + sizeof_len((&self.app_name).len()) }
        + if self.namespace == "" { 0 } else { 1 + sizeof_len((&self.namespace).len()) }
        + if self.name == "" { 0 } else { 1 + sizeof_len((&self.name).len()) }
        + if self.channel_type == "" { 0 } else { 1 + sizeof_len((&self.channel_type).len()) }
        + if self.channel_sub_type == "" { 0 } else { 1 + sizeof_len((&self.channel_sub_type).len()) }
        + if self.url == "" { 0 } else { 1 + sizeof_len((&self.url).len()) }
        + if self.email == "" { 0 } else { 1 + sizeof_len((&self.email).len()) }
        + if self.username == "" { 0 } else { 1 + sizeof_len((&self.username).len()) }
        + if self.password == "" { 0 } else { 1 + sizeof_len((&self.password).len()) }
        + if self.token == "" { 0 } else { 1 + sizeof_len((&self.token).len()) }
        + if self.version_id == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.version_id) as u64) }
        + if self.last_modified_millis == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.last_modified_millis) as u64) }
        + if self.create_time == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.create_time) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.id))?; }
        if self.enable != false { w.write_with_tag(16, |w| w.write_bool(*&self.enable))?; }
        if self.app_name != "" { w.write_with_tag(26, |w| w.write_string(&**&self.app_name))?; }
        if self.namespace != "" { w.write_with_tag(34, |w| w.write_string(&**&self.namespace))?; }
        if self.name != "" { w.write_with_tag(42, |w| w.write_string(&**&self.name))?; }
        if self.channel_type != "" { w.write_with_tag(50, |w| w.write_string(&**&self.channel_type))?; }
        if self.channel_sub_type != "" { w.write_with_tag(58, |w| w.write_string(&**&self.channel_sub_type))?; }
        if self.url != "" { w.write_with_tag(66, |w| w.write_string(&**&self.url))?; }
        if self.email != "" { w.write_with_tag(74, |w| w.write_string(&**&self.email))?; }
        if self.username != "" { w.write_with_tag(82, |w| w.write_string(&**&self.username))?; }
        if self.password != "" { w.write_with_tag(90, |w| w.write_string(&**&self.password))?; }
        if self.token != "" { w.write_with_tag(98, |w| w.write_string(&**&self.token))?; }
        if self.version_id != 0u64 { w.write_with_tag(104, |w| w.write_uint64(*&self.version_id))?; }
        if self.last_modified_millis != 0u64 { w.write_with_tag(112, |w| w.write_uint64(*&self.last_modified_millis))?; }
        if self.create_time != 0u64 { w.write_with_tag(120, |w| w.write_uint64(*&self.create_time))?; }
        Ok(())
    }
}

