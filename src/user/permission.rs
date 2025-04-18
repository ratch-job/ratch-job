/// 权限控制
/// 用户-> 角色 -> 功能模块 -> 权限资源； 从前到后都是一对多；
/// 权限资源分为两类：
/// 1）web资源，由前端控制页面是否支持访问；
/// 2）http请求路径，由后端拦截器控制否支持请求；
use std::{collections::HashSet, hash::Hash, sync::Arc};

use crate::common::constant::{EMPTY_STR, HTTP_METHOD_ALL, HTTP_METHOD_GET};

pub enum Resource {
    WebResource(&'static str),
    Path(&'static str, &'static str),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PathResource {
    pub path: &'static str,
    pub method: &'static str,
}

impl PathResource {
    pub fn match_url(&self, path: &str, method: &str) -> bool {
        let match_method = self.is_match_all_method() || self.method == method;
        if path.is_empty() {
            match_method && (self.is_match_all_path() || self.path == "/")
        } else {
            match_method && (self.is_match_all_path() || self.path == path)
        }
    }

    pub fn is_match_all_path(&self) -> bool {
        self.path == EMPTY_STR
    }
    pub fn is_match_all_method(&self) -> bool {
        self.method == HTTP_METHOD_ALL
    }
}
pub struct ModuleResource {
    pub web_resources: HashSet<&'static str>,
    pub path_resources: HashSet<PathResource>,
}

impl ModuleResource {
    pub fn new(resources: Vec<Resource>) -> Self {
        let mut web_resources = HashSet::new();
        let mut path_resources = HashSet::new();
        for item in resources {
            match item {
                Resource::WebResource(r) => {
                    web_resources.insert(r);
                }
                Resource::Path(path, method) => {
                    path_resources.insert(PathResource { path, method });
                }
            }
        }
        Self {
            web_resources,
            path_resources,
        }
    }

    pub fn match_url(&self, path: &str, method: &str) -> bool {
        for item in &self.path_resources {
            if item.match_url(path, method) {
                return true;
            }
        }
        false
    }
}

pub struct GroupResource {
    pub web_resources: HashSet<&'static str>,
    pub path_resources: HashSet<PathResource>,
}

impl GroupResource {
    pub fn new(module_resources: Vec<&ModuleResource>) -> Self {
        let mut web_resources = HashSet::new();
        let mut path_resources = HashSet::new();
        for module in module_resources {
            for item in &module.web_resources {
                web_resources.insert(*item);
            }
            for item in &module.path_resources {
                path_resources.insert(PathResource { ..(*item) });
            }
        }
        Self {
            web_resources,
            path_resources,
        }
    }

    pub fn match_url(&self, path: &str, method: &str) -> bool {
        for item in &self.path_resources {
            if item.match_url(path, method) {
                return true;
            }
        }
        false
    }
}

type R = Resource;

lazy_static::lazy_static! {
    pub(crate) static ref USER_ROLE_MANAGER: Arc<String> =  Arc::new("0".to_string());
    pub(crate) static ref USER_ROLE_DEVELOPER: Arc<String> =  Arc::new("1".to_string());
    pub(crate) static ref USER_ROLE_VISITOR: Arc<String> =  Arc::new("2".to_string());
    pub(crate) static ref ALL_ROLES: Vec<Arc<String>> = vec![USER_ROLE_MANAGER.clone(),USER_ROLE_DEVELOPER.clone(),USER_ROLE_VISITOR.clone()];
    static ref M_BASE: ModuleResource = ModuleResource::new(vec![
        //WebResource
        R::WebResource("/"),
        R::WebResource("/404"),
        R::WebResource("/nopermission"),
        R::WebResource("/p/login"),
        R::WebResource("/manage/about"),
        R::WebResource("/ratchjob"),
        R::WebResource("/ratchjob/"),
        R::WebResource("/ratchjob/404"),
        R::WebResource("/ratchjob/nopermission"),
        R::WebResource("/ratchjob/p/login"),
        R::WebResource("/ratchjob/manage/about"),
        //path
        R::Path("/",HTTP_METHOD_GET),
        R::Path("/404",HTTP_METHOD_GET),
        R::Path("/nopermission",HTTP_METHOD_GET),
        R::Path("/p/login",HTTP_METHOD_GET),
        R::Path("/manage/about",HTTP_METHOD_GET),
        R::Path("/ratchjob",HTTP_METHOD_GET),
        R::Path("/ratchjob/",HTTP_METHOD_GET),
        R::Path("/ratchjob/404",HTTP_METHOD_GET),
        R::Path("/ratchjob/nopermission",HTTP_METHOD_GET),
        R::Path("/ratchjob/p/login",HTTP_METHOD_GET),
        R::Path("/ratchjob/manage/about",HTTP_METHOD_GET),

        R::Path("/ratchjob/api/console/v1/login/login",HTTP_METHOD_ALL),
        R::Path("/ratchjob/api/console/v1/login/captcha",HTTP_METHOD_ALL),
        R::Path("/ratchjob/api/console/v1/login/logout",HTTP_METHOD_ALL),
        R::Path("/ratchjob/api/console/v1/user/info",HTTP_METHOD_GET),
        R::Path("/ratchjob/api/console/v1/user/web_resources",HTTP_METHOD_GET),
        R::Path("/ratchjob/api/console/v1/user/reset_password",HTTP_METHOD_ALL),
        R::Path("/ratchjob/api/console/v1/namespaces/list",HTTP_METHOD_GET),

    ]);

    static ref M_CLUSTER_VISITOR: ModuleResource = ModuleResource::new(vec![
        //WebResource
        R::WebResource("/manage/cluster"),
        R::WebResource("/ratchjob/manage/cluster"),
        //path
        R::Path("/ratchjob/manage/cluster",HTTP_METHOD_GET),
        R::Path("/ratchjob/api/console/v1/cluster/cluster_node_list",HTTP_METHOD_GET),
    ]);

    static ref M_USER_MANAGE: ModuleResource = ModuleResource::new(vec![
        //WebResource
        R::WebResource("/manage/user"),
        R::WebResource("/ratchjob/manage/user"),
        R::WebResource("USER_UPDATE"),
        //path
        R::Path("/ratchjob/manage/user",HTTP_METHOD_GET),
        R::Path("/ratchjob/api/console/v1/user/list",HTTP_METHOD_GET),
        R::Path("/ratchjob/api/console/v1/user/info",HTTP_METHOD_GET),
        R::Path("/ratchjob/api/console/v1/user/add",HTTP_METHOD_ALL),
        R::Path("/ratchjob/api/console/v1/user/update",HTTP_METHOD_ALL),
        R::Path("/ratchjob/api/console/v1/user/remove",HTTP_METHOD_ALL),

    ]);

    static ref M_METRICS_VISITOR: ModuleResource = ModuleResource::new(vec![
        //WebResource
        R::WebResource("/manage/appmonitor"),
        //path
        R::Path("/ratchjob/manage/appmonitor",HTTP_METHOD_GET),
        R::Path("/ratchjob/api/console/v1/metrics/timeline",HTTP_METHOD_ALL),
        R::Path("/ratchjob/api/console/v1/cluster/cluster_node_list",HTTP_METHOD_GET),
    ]);


    static ref R_VISITOR: Arc<GroupResource> = Arc::new(GroupResource::new(vec![
        &M_BASE,
        //&M_CLUSTER_VISITOR,
        //&M_NAMESPACE_VISITOR,
    ]));

    static ref R_DEVELOPER: Arc<GroupResource> = Arc::new(GroupResource::new(vec![
        &M_BASE,
        &M_CLUSTER_VISITOR,
        &M_METRICS_VISITOR,
    ]));

    static ref R_MANAGER: Arc<GroupResource> = Arc::new(GroupResource::new(vec![
        &M_BASE,
        &M_CLUSTER_VISITOR,
        &M_USER_MANAGE,
        &M_METRICS_VISITOR,
    ]));

}

#[derive(Debug)]
pub enum UserRole {
    Visitor,
    Developer,
    Manager,
    None,
}
const MANAGER_VALUE: &str = "0";
const DEVELOPER_VALUE: &str = "1";
const VISITOR_VALUE: &str = "2";
const NONE_VALUE: &str = "";

impl UserRole {
    pub fn new(role_value: &str) -> Self {
        match role_value {
            MANAGER_VALUE => Self::Manager,
            DEVELOPER_VALUE => Self::Developer,
            VISITOR_VALUE => Self::Visitor,
            _ => Self::None,
        }
    }

    pub fn to_role_value(&self) -> &str {
        match self {
            Self::Manager => MANAGER_VALUE,
            Self::Developer => DEVELOPER_VALUE,
            Self::Visitor => VISITOR_VALUE,
            _ => NONE_VALUE,
        }
    }

    pub fn get_resources(&self) -> Vec<&GroupResource> {
        match &self {
            UserRole::Visitor => vec![R_VISITOR.as_ref()],
            UserRole::Developer => vec![R_DEVELOPER.as_ref()],
            UserRole::Manager => vec![R_MANAGER.as_ref()],
            UserRole::None => vec![],
        }
    }

    pub fn match_url(&self, path: &str, method: &str) -> bool {
        for item in self.get_resources() {
            if item.match_url(path, method) {
                return true;
            }
        }
        false
    }

    pub fn match_url_by_roles(role_values: &Vec<Arc<String>>, path: &str, method: &str) -> bool {
        for item in role_values {
            if Self::new(item.as_str()).match_url(path, method) {
                return true;
            }
        }
        false
    }

    pub fn get_web_resources(&self) -> Vec<&'static str> {
        //log::info!("get_web_resources {:?}", &self);
        let resources = self.get_resources();
        if resources.len() == 1 {
            return resources
                .first()
                .unwrap()
                .web_resources
                .iter()
                .copied()
                .collect();
        }
        let mut set = HashSet::new();
        for resource in resources {
            for item in &resource.web_resources {
                set.insert(*item);
            }
        }
        set.into_iter().collect()
    }

    pub fn get_web_resources_by_roles(role_values: Vec<&str>) -> Vec<&'static str> {
        //log::info!("get_web_resources_by_roles {:?}", &role_values);
        let roles: Vec<Self> = role_values.into_iter().map(Self::new).collect();
        if roles.len() == 1 {
            return roles.first().unwrap().get_web_resources();
        }
        let mut set = HashSet::new();
        for role in roles {
            for resource in role.get_resources() {
                for item in &resource.web_resources {
                    set.insert(*item);
                }
            }
        }
        set.into_iter().collect()
    }
}

pub struct UserRoleHelper;

impl UserRoleHelper {
    pub fn get_all_roles() -> Vec<Arc<String>> {
        ALL_ROLES.clone()
    }

    pub fn get_role(role_value: &str) -> Arc<String> {
        for item in ALL_ROLES.iter() {
            if role_value == item.as_str() {
                return item.clone();
            }
        }
        Arc::new(role_value.to_owned())
    }
}
