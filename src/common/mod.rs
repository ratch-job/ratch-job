pub mod app_config;
pub mod share_data;

pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
