use crate::app::model::AppKey;
use crate::common::app_config::AppConfig;
use crate::common::get_app_version;
use crate::job::model::job::JobInfo;
use crate::task::model::actor_model::{TaskManagerReq, TaskManagerResult};
use crate::task::model::app_instance::{AppInstanceStateGroup, InstanceAddrSelectResult};
use crate::task::model::request_model::JobRunParam;
use crate::task::model::task::JobTaskInfo;
use crate::task::request_client::XxlClient;
use actix::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TaskManager {
    task_instance_map: HashMap<u64, JobTaskInfo>,
    app_instance_group: HashMap<AppKey, AppInstanceStateGroup>,
    xxl_request_header: HashMap<String, String>,
}

impl TaskManager {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let mut xxl_request_header = HashMap::new();
        xxl_request_header.insert("Content-Type".to_string(), "application/json".to_string());
        xxl_request_header.insert(
            "User-Agent".to_owned(),
            format!("ratch-job/{}", get_app_version()),
        );
        if !config.xxl_default_access_token.is_empty() {
            xxl_request_header.insert(
                "XXL-JOB-ACCESS-TOKEN".to_owned(),
                config.xxl_default_access_token.clone(),
            );
        }
        TaskManager {
            task_instance_map: HashMap::new(),
            app_instance_group: HashMap::new(),
            xxl_request_header: HashMap::new(),
        }
    }

    pub fn add_app_instance(&mut self, app_key: AppKey, instance_addr: Arc<String>) {
        if let Some(app_instance_group) = self.app_instance_group.get_mut(&app_key) {
            app_instance_group.add_instance(instance_addr);
        } else {
            let mut app_instance_group = AppInstanceStateGroup::new(app_key.clone());
            app_instance_group.add_instance(instance_addr);
            self.app_instance_group.insert(app_key, app_instance_group);
        }
    }

    pub fn remove_app_instance(&mut self, app_key: AppKey, instance_addr: Arc<String>) {
        if let Some(app_instance_group) = self.app_instance_group.get_mut(&app_key) {
            app_instance_group.remove_instance(instance_addr);
        }
    }

    fn trigger_task(
        &mut self,
        trigger_time: u32,
        job_info: Arc<JobInfo>,
        ctx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let app_key = AppKey::new(job_info.app_name.clone(), job_info.namespace.clone());
        if let Some(app_instance_group) = self.app_instance_group.get_mut(&app_key) {
            let select = app_instance_group.select_instance(&job_info.router_strategy, job_info.id);
            Self::run_task(
                trigger_time,
                job_info,
                select,
                app_instance_group.instance_keys.clone(),
                self.xxl_request_header.clone(),
            )
            .into_actor(self)
            .map(|r, _act, _ctx| match r {
                Ok(_) => {
                    log::info!("run task success")
                }
                Err(e) => {
                    log::error!("run task error:{}", e)
                }
            })
            .spawn(ctx);
        }
        Ok(())
    }

    async fn run_task(
        trigger_time: u32,
        job_info: Arc<JobInfo>,
        select_instance: InstanceAddrSelectResult,
        addrs: Vec<Arc<String>>,
        xxl_request_header: HashMap<String, String>,
    ) -> anyhow::Result<JobTaskInfo> {
        let mut task_instance = JobTaskInfo::from_job(trigger_time, &job_info);
        let client = reqwest::Client::new();
        //todo 获取日志id
        let task_id = 1;
        task_instance.task_id = task_id;
        let param = JobRunParam::from_job_info(task_id, &job_info);
        match select_instance {
            InstanceAddrSelectResult::Selected(addr) => {
                //todo 重试
                Self::do_run_task(addr, &param, &client, &xxl_request_header).await?;
            }
            InstanceAddrSelectResult::ALL(addrs) => {
                for addr in addrs {
                    Self::do_run_task(addr, &param, &client, &xxl_request_header).await?;
                }
            }
            InstanceAddrSelectResult::Empty => {}
        }
        Ok(task_instance)
    }
    async fn do_run_task(
        instance_addr: Arc<String>,
        param: &JobRunParam,
        client: &reqwest::Client,
        xxl_request_header: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let xxl_client = XxlClient::new(&client, &xxl_request_header, &instance_addr);
        xxl_client.run_job(param).await?;
        Ok(())
    }
}

impl Actor for TaskManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("TaskManager started")
    }
}

impl Handler<TaskManagerReq> for TaskManager {
    type Result = anyhow::Result<TaskManagerResult>;

    fn handle(&mut self, msg: TaskManagerReq, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TaskManagerReq::AddAppInstance(app_key, instance_addr) => {
                self.add_app_instance(app_key, instance_addr);
            }
            TaskManagerReq::RemoveAppInstance(app_key, instance_addr) => {
                self.remove_app_instance(app_key, instance_addr);
            }
            TaskManagerReq::TriggerTask(trigger_time, job) => {
                self.trigger_task(trigger_time, job, ctx)?;
            }
        }
        Ok(TaskManagerResult::None)
    }
}
