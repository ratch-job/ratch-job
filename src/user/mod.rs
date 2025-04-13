pub mod actor_model;
pub mod core;
pub mod model;

pub(crate) fn build_password_hash(password: &str) -> anyhow::Result<String> {
    Ok(bcrypt::hash(password, 10u32)?)
}
