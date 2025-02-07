use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppQueryParam {
    pub app_name: Option<String>,
    pub namespace: Option<String>,
}
