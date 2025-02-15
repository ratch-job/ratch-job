use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceInfo {
    pub namespace_id: Option<Arc<String>>,
    pub namespace_name: Option<String>,
    pub r#type: Option<String>,
}