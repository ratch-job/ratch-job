pub mod actor_utils;
pub mod app_config;
pub mod byte_utils;
pub mod constant;
pub mod cron_utils;
pub mod datetime_utils;
pub mod hash_utils;
pub mod http_utils;
pub mod model;
pub mod namespace_util;
pub mod option_utils;
pub mod pb;
pub mod protobuf_utils;
pub mod sequence_utils;
pub mod share_data;
pub mod string_utils;
pub mod tempfile;

pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
