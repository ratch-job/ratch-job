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
    pub register_time: u64,
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
                Ok(160) => msg.register_time = r.read_uint64(bytes)?,
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
        + if self.register_time == 0u64 { 0 } else { 2 + sizeof_varint(*(&self.register_time) as u64) }
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
        if self.register_time != 0u64 { w.write_with_tag(160, |w| w.write_uint64(*&self.register_time))?; }
        Ok(())
    }
}

