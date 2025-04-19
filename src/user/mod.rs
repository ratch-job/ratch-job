pub mod actor_model;
pub mod core;
pub mod model;
pub mod permission;

pub(crate) fn build_password_hash(password: &str) -> anyhow::Result<String> {
    Ok(bcrypt::hash(password, 10u32)?)
}

pub(crate) fn verify_password_hash(password: &str, password_hash: &str) -> anyhow::Result<bool> {
    Ok(bcrypt::verify(password, password_hash)?)
}

/*
pub(crate) fn verify_password_hash_option(
    password: &str,
    password_hash: &Option<String>,
) -> anyhow::Result<bool> {
    if let Some(password_hash) = password_hash {
        verify_password_hash(password, password_hash)
    } else {
        Err(anyhow::anyhow!("password_hash is empty"))
    }
}
*/
