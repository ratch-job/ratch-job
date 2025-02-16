use crate::app::model::AppKey;
use crate::common::app_config::AppConfig;
use crate::common::constant::SEQ_TASK_ID;
use crate::common::get_app_version;
use crate::job::core::JobManager;
use crate::job::model::actor_model::JobManagerReq;
use crate::job::model::job::JobInfo;
use crate::sequence::{SequenceManager, SequenceRequest, SequenceResult};
use crate::task::model::actor_model::{TaskCallBackParam, TaskManagerReq, TaskManagerResult};
use crate::task::model::app_instance::{AppInstanceStateGroup, InstanceAddrSelectResult};
use crate::task::model::enum_type::TaskStatusType;
use crate::task::model::request_model::JobRunParam;
use crate::task::model::task::JobTaskInfo;
use crate::task::request_client::XxlClient;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use std::collections::HashMap;
use std::sync::Arc;

#[bean(inject)]
pub struct TaskManager {
    task_instance_map: HashMap<u64, JobTaskInfo>,
    app_instance_group: HashMap<AppKey, AppInstanceStateGroup>,
    xxl_request_header: HashMap<String, String>,
    sequence_manager: Option<Addr<SequenceManager>>,
    job_manager: Option<Addr<JobManager>>,
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
            xxl_request_header,
            sequence_manager: None,
            job_manager: None,
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
        if self.sequence_manager.is_none() || self.job_manager.is_none() {
            return Err(anyhow::anyhow!("sequence_manager or job_manager is none"));
        }
        let sequence_manager = self.sequence_manager.clone().unwrap();
        let job_manager = self.job_manager.clone().unwrap();
        if let Some(app_instance_group) = self.app_instance_group.get_mut(&app_key) {
            let select = app_instance_group.select_instance(&job_info.router_strategy, job_info.id);
            Self::run_task(
                trigger_time,
                job_info,
                select,
                app_instance_group.instance_keys.clone(),
                self.xxl_request_header.clone(),
                sequence_manager,
                job_manager,
            )
            .into_actor(self)
            .map(|task_info, act, _ctx| {
                if task_info.status == TaskStatusType::Running {
                    log::info!(
                        "run task Running,job_id:{},task_id:{}",
                        &task_info.job_id,
                        &task_info.task_id
                    );
                }
                act.task_instance_map.insert(task_info.task_id, task_info);
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
        sequence_manager: Addr<SequenceManager>,
        job_manager: Addr<JobManager>,
    ) -> JobTaskInfo {
        let mut task_instance = JobTaskInfo::from_job(trigger_time, &job_info);
        let client = reqwest::Client::new();
        let task_id = if let Ok(Ok(SequenceResult::NextId(task_id))) = sequence_manager
            .send(SequenceRequest::GetNextId(SEQ_TASK_ID.clone()))
            .await
        {
            task_id
        } else {
            log::error!("get task id error!");
            task_instance.status = TaskStatusType::Error;
            task_instance.trigger_message = Arc::new("get task id error!".to_string());
            job_manager.do_send(JobManagerReq::UpdateTask(Arc::new(task_instance.clone())));
            return task_instance;
        };
        task_instance.task_id = task_id;
        let mut param = JobRunParam::from_job_info(task_id, &job_info);
        param.log_date_time = Some(trigger_time as u64 * 1000);
        task_instance.status = TaskStatusType::Running;
        if let InstanceAddrSelectResult::Selected(addr) = &select_instance {
            task_instance.instance_addr = addr.clone();
        }
        job_manager.do_send(JobManagerReq::UpdateTask(Arc::new(task_instance.clone())));
        match select_instance {
            InstanceAddrSelectResult::Selected(addr) => {
                if Self::do_run_task(addr, &param, &client, &xxl_request_header)
                    .await
                    .is_err()
                {
                    //todo 重试
                    task_instance.status = TaskStatusType::Error;
                }
            }
            InstanceAddrSelectResult::ALL(addrs) => {
                for addr in addrs {
                    Self::do_run_task(addr, &param, &client, &xxl_request_header)
                        .await
                        .ok();
                }
            }
            InstanceAddrSelectResult::Empty => {}
        }
        job_manager.do_send(JobManagerReq::UpdateTask(Arc::new(task_instance.clone())));
        task_instance
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

    fn task_callback(&mut self, params: Vec<TaskCallBackParam>) -> anyhow::Result<()> {
        if self.job_manager.is_none() {
            return Err(anyhow::anyhow!("job_manager is none"));
        }
        let job_manager = self.job_manager.clone().unwrap();
        for param in params {
            if let Some(mut task_instance) = self.task_instance_map.remove(&param.task_id) {
                if param.success {
                    task_instance.status = TaskStatusType::Success;
                } else {
                    task_instance.status = TaskStatusType::Error;
                    if let Some(msg) = param.handle_msg {
                        task_instance.callback_message = Arc::new(msg);
                    }
                }
                job_manager.do_send(JobManagerReq::UpdateTask(Arc::new(task_instance)));
            }
        }
        Ok(())
    }
}

impl Actor for TaskManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("TaskManager started")
    }
}

impl Inject for TaskManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.sequence_manager = factory_data.get_actor();
        self.job_manager = factory_data.get_actor();
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
            TaskManagerReq::TaskCallBacks(params) => {
                self.task_callback(params)?;
            }
        }
        Ok(TaskManagerResult::None)
    }
}
