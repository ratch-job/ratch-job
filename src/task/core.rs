use crate::app::model::AppKey;
use crate::common::app_config::AppConfig;
use crate::common::constant::{ERR_MSG_NOT_FOUND_APP_INSTANCE_ADDR, SEQ_TASK_ID};
use crate::common::datetime_utils::now_second_u32;
use crate::common::get_app_version;
use crate::job::core::JobManager;
use crate::job::model::actor_model::{JobManagerRaftReq, JobManagerReq};
use crate::job::model::job::JobInfo;
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::ClientRequest;
use crate::sequence::model::SeqRange;
use crate::sequence::{SequenceManager, SequenceRequest, SequenceResult};
use crate::task::model::actor_model::{
    TaskManagerReq, TaskManagerResult, TriggerItem, TriggerSourceInfo, TriggerSourceType,
};
use crate::task::model::app_instance::{AppInstanceStateGroup, InstanceAddrSelectResult};
use crate::task::model::enum_type::TaskStatusType;
use crate::task::model::request_model::JobRunParam;
use crate::task::model::task::{JobTaskInfo, TaskCallBackParam, TaskWrap};
use crate::task::request_client::XxlClient;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use std::collections::HashMap;
use std::sync::Arc;

#[bean(inject)]
pub struct TaskManager {
    app_instance_group: HashMap<AppKey, AppInstanceStateGroup>,
    xxl_request_header: HashMap<String, String>,
    sequence_manager: Option<Addr<SequenceManager>>,
    job_manager: Option<Addr<JobManager>>,
    raft_request_route: Option<Arc<RaftRequestRoute>>,
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
            app_instance_group: HashMap::new(),
            xxl_request_header,
            sequence_manager: None,
            job_manager: None,
            raft_request_route: None,
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

    fn trigger_task_list(
        &mut self,
        trigger_items: Vec<TriggerItem>,
        ctx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        if trigger_items.is_empty() {
            return Ok(());
        }
        if self.sequence_manager.is_none() || self.raft_request_route.is_none() {
            log::error!("sequence_manager or job_manager is none");
            return Err(anyhow::anyhow!("sequence_manager or job_manager is none"));
        }
        let sequence_manager = self.sequence_manager.clone().unwrap();
        let raft_request_route = self.raft_request_route.clone().unwrap();
        Self::init_tasks(trigger_items, raft_request_route, sequence_manager)
            .into_actor(self)
            .then(|result, act, ctx| {
                let list = result.unwrap_or_default();
                let (task_list, notify_task_list) = act.build_task_wrap(list);
                let raft_request_route = act.raft_request_route.clone().unwrap();
                let xxl_request_header = act.xxl_request_header.clone();
                async move {
                    Self::notify_update_task(&raft_request_route, notify_task_list).await?;
                    Self::run_task_list(task_list, xxl_request_header, raft_request_route).await?;
                    Ok(())
                }
                .into_actor(act)
            })
            .map(|_: anyhow::Result<()>, _, _| ())
            .spawn(ctx);
        Ok(())
    }

    async fn init_tasks(
        trigger_items: Vec<TriggerItem>,
        raft_request_route: Arc<RaftRequestRoute>,
        sequence_manager: Addr<SequenceManager>,
    ) -> anyhow::Result<Vec<(JobTaskInfo, Arc<JobInfo>, TriggerSourceInfo)>> {
        let range = Self::fetch_task_ids(sequence_manager, trigger_items.len() as u64).await?;
        let mut start_id = range.start;
        let mut task_list = Vec::with_capacity(trigger_items.len());
        let mut notify_task_list = Vec::with_capacity(trigger_items.len());
        for item in trigger_items {
            let mut task_instance = JobTaskInfo::from_job(item.trigger_time, &item.job_info);
            if let TriggerSourceType::User(_) = &item.trigger_source.source_type {
                task_instance.try_times = 0;
            }
            task_instance.task_id = start_id;
            start_id += 1;
            task_instance.status = TaskStatusType::Init;
            task_instance.trigger_from = item.trigger_source.source_type.get_source_from();
            notify_task_list.push(Arc::new(task_instance.clone()));
            task_list.push((task_instance, item.job_info, item.trigger_source))
        }
        Self::notify_update_task(&raft_request_route, notify_task_list).await?;
        Ok(task_list)
    }

    async fn fetch_task_ids(
        sequence_manager: Addr<SequenceManager>,
        len: u64,
    ) -> anyhow::Result<SeqRange> {
        let res = sequence_manager
            .send(SequenceRequest::GetDirectRange(SEQ_TASK_ID.clone(), len))
            .await??;
        if let SequenceResult::Range(range) = res {
            Ok(range)
        } else {
            log::error!("sequence_manager get direct range error");
            Err(anyhow::anyhow!("sequence_manager get direct range error"))
        }
    }
    async fn notify_update_task(
        raft_request_route: &Arc<RaftRequestRoute>,
        tasks: Vec<Arc<JobTaskInfo>>,
    ) -> anyhow::Result<()> {
        if tasks.is_empty() {
            return Ok(());
        }
        raft_request_route
            .request(ClientRequest::JobReq {
                req: JobManagerRaftReq::UpdateTaskList(tasks),
            })
            .await?;
        Ok(())
    }

    fn build_task_wrap(
        &mut self,
        tasks: Vec<(JobTaskInfo, Arc<JobInfo>, TriggerSourceInfo)>,
    ) -> (Vec<TaskWrap>, Vec<Arc<JobTaskInfo>>) {
        let mut task_list = Vec::with_capacity(tasks.len());
        let mut ignore_task_list = Vec::new();
        let now_second = now_second_u32();
        for (mut task, job_info, trigger_source) in tasks {
            task.execution_time = now_second;
            let app_key = job_info.build_app_key();
            if let Some(app_instance_group) = self.app_instance_group.get_mut(&app_key) {
                let select = if trigger_source.fix_addr.is_empty() {
                    app_instance_group.select_instance(&job_info.router_strategy, job_info.id)
                } else {
                    InstanceAddrSelectResult::Fixed(trigger_source.fix_addr.clone())
                };
                if let &InstanceAddrSelectResult::Empty = &select {
                    task.status = TaskStatusType::Error;
                    task.finish_time = now_second;
                    task.trigger_message = ERR_MSG_NOT_FOUND_APP_INSTANCE_ADDR.clone();
                    ignore_task_list.push(Arc::new(task));
                } else {
                    let wrap = TaskWrap {
                        task,
                        job_info,
                        select_result: select,
                        app_addrs: app_instance_group.instance_keys.clone(),
                        trigger_source,
                    };
                    task_list.push(wrap);
                }
            } else {
                task.status = TaskStatusType::Error;
                task.finish_time = now_second;
                task.trigger_message = ERR_MSG_NOT_FOUND_APP_INSTANCE_ADDR.clone();
                ignore_task_list.push(Arc::new(task));
            }
        }
        (task_list, ignore_task_list)
    }

    async fn run_task_list(
        task_wrap_list: Vec<TaskWrap>,
        xxl_request_header: HashMap<String, String>,
        raft_request_route: Arc<RaftRequestRoute>,
    ) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let mut task_list = Vec::with_capacity(task_wrap_list.len());
        for task_wrap in task_wrap_list {
            let mut task_info = task_wrap.task;
            let mut param = JobRunParam::from_job_info(task_info.task_id, &task_wrap.job_info);
            param.log_date_time = Some(task_info.trigger_time as u64 * 1000);
            match task_wrap.select_result {
                InstanceAddrSelectResult::Fixed(addr) => {
                    task_info.instance_addr = addr.clone();
                    if let Err(err) =
                        Self::do_run_task(addr, &param, &client, &xxl_request_header).await
                    {
                        task_info.status = TaskStatusType::Error;
                        task_info.trigger_message = Arc::new(err.to_string());
                        task_info.finish_time = now_second_u32();
                        log::error!("run task error:{}", err);
                    } else {
                        task_info.status = TaskStatusType::Running;
                    }
                }
                InstanceAddrSelectResult::Selected(addr) => {
                    task_info.instance_addr = addr.clone();
                    //let try_times = std::cmp::max(std::cmp::min(task_wrap.job_info.try_times, 5u32),1u32);
                    //if let Err(err) = Self::try_run_task( addr, task_wrap.app_addrs, &param, &client, &xxl_request_header, try_times).await
                    if let Err(err) =
                        Self::do_run_task(addr, &param, &client, &xxl_request_header).await
                    {
                        task_info.status = TaskStatusType::Error;
                        task_info.trigger_message = Arc::new(err.to_string());
                        task_info.finish_time = now_second_u32();
                        //log::error!("run task error,try_times:{},err:{}", try_times, err);
                        log::error!("run task error,err:{}", err);
                    } else {
                        task_info.status = TaskStatusType::Running;
                    }
                }
                InstanceAddrSelectResult::ALL(addrs) => {
                    for addr in addrs {
                        Self::do_run_task(addr, &param, &client, &xxl_request_header)
                            .await
                            .ok();
                    }
                    task_info.status = TaskStatusType::Running;
                }
                InstanceAddrSelectResult::Empty => {
                    //前面已处理过，不会执行到这里
                }
            }
            task_list.push(Arc::new(task_info));
        }
        Self::notify_update_task(&raft_request_route, task_list).await?;
        Ok(())
    }

    /*
    async fn try_run_task(
        instance_addr: Arc<String>,
        all_addr: Vec<Arc<String>>,
        param: &JobRunParam,
        client: &reqwest::Client,
        xxl_request_header: &HashMap<String, String>,
        try_times: u32,
    ) -> anyhow::Result<()> {
        let mut times = try_times;
        let mut index = 0;
        if times > 0 {
            for (i, addr) in all_addr.iter().enumerate() {
                if addr == &instance_addr {
                    index = i;
                    break;
                }
            }
        }
        let mut current_addr = instance_addr;
        while times >= 0 {
            if let Err(err) =
                Self::do_run_task(current_addr.clone(), param, client, xxl_request_header).await
            {
                if times == 0 || all_addr.is_empty() {
                    return Err(err);
                }
                //获取下一个地址
                index += 1;
                current_addr = if index == all_addr.len() {
                    all_addr[0].clone()
                } else {
                    all_addr[index].clone()
                };
                log::warn!("try run task error,last times:{},err:{}", times, err);
                times -= 1;
            } else {
                return Ok(());
            }
        }
        Ok(())
    }
     */

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
        self.raft_request_route = factory_data.get_bean();
    }
}

impl Handler<TaskManagerReq> for TaskManager {
    type Result = anyhow::Result<TaskManagerResult>;

    fn handle(&mut self, msg: TaskManagerReq, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TaskManagerReq::AddAppInstance(app_key, instance_addr) => {
                self.add_app_instance(app_key, instance_addr);
            }
            TaskManagerReq::AddAppInstances(app_instance_keys) => {
                for keys in app_instance_keys {
                    self.add_app_instance(keys.build_app_key(), keys.addr);
                }
            }
            TaskManagerReq::RemoveAppInstance(app_key, instance_addr) => {
                self.remove_app_instance(app_key, instance_addr);
            }
            TaskManagerReq::RemoveAppInstances(app_instance_keys) => {
                for keys in app_instance_keys {
                    self.remove_app_instance(keys.build_app_key(), keys.addr);
                }
            }
            TaskManagerReq::TriggerTaskList(trigger_list) => {
                self.trigger_task_list(trigger_list, ctx)?;
            }
        }
        Ok(TaskManagerResult::None)
    }
}
