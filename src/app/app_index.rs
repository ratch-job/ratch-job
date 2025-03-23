use crate::app::model::AppKey;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppQueryParam {
    pub namespace: Option<Arc<String>>,
    pub app_name: Option<Arc<String>>,
    pub like_name: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

impl AppQueryParam {
    pub fn match_namespace(&self, value: &Arc<String>) -> bool {
        if let Some(namespace) = &self.namespace {
            namespace.is_empty() || namespace == value
        } else {
            true
        }
    }

    pub fn match_app_name(&self, value: &Arc<String>) -> bool {
        if let Some(app_name) = &self.app_name {
            app_name.is_empty() || app_name == value
        } else if let Some(like_name) = &self.like_name {
            like_name.is_empty() || value.contains(like_name)
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppIndex {
    pub(crate) group_data: BTreeMap<Arc<String>, BTreeSet<Arc<String>>>,
}

impl AppIndex {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn insert(&mut self, namespace: Arc<String>, app_name: Arc<String>) -> bool {
        if let Some(group_set) = self.group_data.get_mut(&namespace) {
            group_set.insert(app_name)
        } else {
            let mut group_set = BTreeSet::new();
            group_set.insert(app_name);
            self.group_data.insert(namespace, group_set);
            true
        }
    }

    pub(crate) fn remove(
        &mut self,
        namespace: &Arc<String>,
        app_name: &Arc<String>,
    ) -> (bool, usize) {
        if let Some(group_set) = self.group_data.get_mut(namespace) {
            let b = group_set.remove(app_name);
            if b && group_set.is_empty() {
                self.group_data.remove(namespace);
            }
            (b, self.group_data.len())
        } else {
            (false, self.group_data.len())
        }
    }

    pub fn query(&self, param: &AppQueryParam) -> (usize, Vec<AppKey>) {
        let mut rlist = Vec::new();
        let end_index = param.offset + param.limit;
        let mut index = 0;
        for (namespace, group_set) in self.group_data.iter() {
            if param.match_namespace(namespace) {
                for app_name in group_set.iter() {
                    if param.match_app_name(app_name) {
                        if index >= param.offset && index < end_index {
                            rlist.push(AppKey::new(app_name.clone(), namespace.clone()));
                        }
                        index += 1;
                    }
                }
            }
        }
        (index, rlist)
    }

    pub fn get_namespace_count(&self) -> usize {
        self.group_data.len()
    }

    pub fn get_item_count(&self) -> usize {
        self.group_data.values().map(|set| set.len()).sum()
    }
}
