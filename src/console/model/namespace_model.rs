use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceInfo {
    pub namespace_id: Option<Arc<String>>,
    pub namespace_name: Option<String>,
    pub r#type: Option<String>,
}
