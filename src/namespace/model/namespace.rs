use crate::common::pb::data_object::NamespaceDo;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Namespace {
    pub id: String,
    pub name: String,
    pub r#type: String,
}

impl Namespace {
    pub fn to_do(&self) -> NamespaceDo<'_> {
        NamespaceDo {
            id: Cow::Borrowed(&self.id),
            name: Cow::Borrowed(&self.name),
            type_pb: Cow::Borrowed(&self.r#type),
        }
    }
}

impl<'a> From<NamespaceDo<'a>> for Namespace {
    fn from(ns_do: NamespaceDo<'a>) -> Self {
        Namespace {
            id: ns_do.id.to_string(),
            name: ns_do.name.to_string(),
            r#type: ns_do.type_pb.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceParam {
    pub id: Option<String>,
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceInfo {
    pub id: Arc<String>,
    pub name: String,
    pub r#type: String,
}

impl NamespaceInfo {
    pub fn from_namespace(ns: &Namespace) -> Self {
        NamespaceInfo {
            id: Arc::new(ns.id.clone()),
            name: ns.name.clone(),
            r#type: ns.r#type.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceQueryParam {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub r#type: Option<String>,
}

pub struct NamespaceWrap {
    pub namespace: Arc<Namespace>,
    pub name_lower: String,
    pub type_lower: String,
}

impl NamespaceWrap {
    pub fn new(namespace: Arc<Namespace>) -> Self {
        let name_lower = namespace.name.to_lowercase();
        let type_lower = namespace.r#type.to_lowercase();
        NamespaceWrap {
            namespace,
            name_lower,
            type_lower,
        }
    }
}
