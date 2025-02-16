use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobQueryParam {
    pub namespace: Option<Arc<String>>,
    pub app_name: Option<Arc<String>>,
    pub like_description: Option<Arc<String>>,
    pub like_handle_name: Option<Arc<String>>,
    pub offset: usize,
    pub limit: usize,
}

impl JobQueryParam {
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
        } else {
            true
        }
    }

    pub fn match_description(&self, value: &Arc<String>) -> bool {
        if let Some(like_description) = &self.like_description {
            like_description.is_empty() || value.as_str().contains(like_description.as_str())
        } else {
            true
        }
    }

    pub fn match_handle_name(&self, value: &Arc<String>) -> bool {
        if let Some(like_handle_name) = &self.like_handle_name {
            like_handle_name.is_empty() || value.as_str().contains(like_handle_name.as_str())
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JobIndexInfo {
    pub job_id: u64,
    pub description: Arc<String>,
    pub handle_name: Arc<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JobIndex {
    pub(crate) group_data:
        BTreeMap<Arc<String>, BTreeMap<Arc<String>, BTreeMap<u64, JobIndexInfo>>>,
}

impl JobIndex {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn insert(
        &mut self,
        namespace: Arc<String>,
        app_name: Arc<String>,
        job_info: JobIndexInfo,
    ) -> bool {
        self.group_data
            .entry(namespace)
            .or_insert_with(BTreeMap::new)
            .entry(app_name)
            .or_insert_with(BTreeMap::new)
            .insert(job_info.job_id, job_info)
            .is_none()
    }

    pub(crate) fn remove(
        &mut self,
        namespace: &Arc<String>,
        app_name: &Arc<String>,
        job_id: &u64,
    ) -> bool {
        if let Some(app_map) = self.group_data.get_mut(namespace) {
            if let Some(job_map) = app_map.get_mut(app_name) {
                let b = job_map.remove(job_id).is_some();
                if b && job_map.is_empty() {
                    app_map.remove(app_name);
                }
                if app_map.is_empty() {
                    self.group_data.remove(namespace);
                }
                b
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn query(&self, param: &JobQueryParam) -> (usize, Vec<JobIndexInfo>) {
        let mut rlist = Vec::new();
        let end_index = param.offset + param.limit;
        let mut index = 0;

        for (namespace, app_map) in self.group_data.iter() {
            if param.match_namespace(namespace) {
                for (app_name, job_map) in app_map.iter() {
                    if param.match_app_name(app_name) {
                        for job_info in job_map.values() {
                            if param.match_description(&job_info.description)
                                && param.match_handle_name(&job_info.handle_name)
                            {
                                if index >= param.offset && index < end_index {
                                    rlist.push(job_info.clone());
                                }
                                index += 1;
                            }
                        }
                    }
                }
            }
        }

        (index, rlist)
    }

    pub fn get_namespace_count(&self) -> usize {
        self.group_data.len()
    }

    pub fn get_app_count(&self) -> usize {
        self.group_data.values().map(|app_map| app_map.len()).sum()
    }

    pub fn get_item_count(&self) -> usize {
        self.group_data
            .values()
            .map(|app_map| app_map.values().map(|job_map| job_map.len()).sum::<usize>())
            .sum()
    }
}
