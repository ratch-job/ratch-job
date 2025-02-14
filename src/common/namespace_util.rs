use crate::common::constant::DEFAULT_XXL_NAMESPACE;
use std::sync::Arc;

pub fn get_namespace(namespace: &Arc<String>) -> Arc<String> {
    if namespace.is_empty() {
        DEFAULT_XXL_NAMESPACE.clone()
    } else {
        namespace.clone()
    }
}

pub fn get_namespace_by_option(namespace: &Option<Arc<String>>) -> Arc<String> {
    if let Some(namespace) = namespace {
        get_namespace(namespace)
    } else {
        DEFAULT_XXL_NAMESPACE.clone()
    }
}
