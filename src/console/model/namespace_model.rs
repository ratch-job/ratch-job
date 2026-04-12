use crate::namespace::model::namespace::NamespaceParam;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceInfo {
    pub namespace_id: Option<Arc<String>>,
    pub namespace_name: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceHandleRequest {
    pub namespace_id: Option<Arc<String>>,
    pub namespace_name: Option<String>,
}

impl NamespaceHandleRequest {
    pub fn to_param(self) -> NamespaceParam {
        NamespaceParam {
            id: self.namespace_id,
            name: self.namespace_name.unwrap_or_default(),
        }
    }
}
