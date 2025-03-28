use crate::common::app_config::AppConfig;
use crate::common::datetime_utils::now_second_u32;
use crate::common::get_app_version;
use crate::job::model::actor_model::JobManagerRaftReq;
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::ClientRequest;
use crate::schedule::batch_call::{BatchCallManager, BatchUpdateTaskManagerReq};
use crate::task::model::enum_type::TaskStatusType;
use crate::task::model::request_model::JobRunParam;
use crate::task::model::task::JobTaskInfo;
use crate::task::model::task_request::{TaskRequestCmd, TaskRequestResult};
use crate::task::request_client::XxlClient;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use std::collections::HashMap;
use std::sync::Arc;

#[bean(inject)]
#[derive(Clone)]
pub struct TaskRequestActor {
    client: reqwest::Client,
    xxl_request_header: HashMap<String, String>,
    batch_call_manager: Option<Addr<BatchCallManager>>,
    request_semaphore: Arc<tokio::sync::Semaphore>,
    pub(crate) running_count: usize,
}

impl TaskRequestActor {
    pub(crate) fn new(config: Arc<AppConfig>) -> Self {
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
        let client = reqwest::Client::new();
        Self {
            client,
            xxl_request_header,
            batch_call_manager: None,
            request_semaphore: Arc::new(tokio::sync::Semaphore::new(config.task_request_parallel)),
            running_count: 0,
        }
    }

    async fn async_run_task(
        msg: TaskRequestCmd,
        xxl_request_header: HashMap<String, String>,
        client: reqwest::Client,
        semaphore: Arc<tokio::sync::Semaphore>,
    ) -> anyhow::Result<(anyhow::Result<()>, Option<JobTaskInfo>)> {
        let permit = match semaphore.acquire_owned().await {
            Ok(permit) => permit,
            Err(err) => {
                return Ok((Err(err.into()), msg.get_task()));
            }
        };
        match msg {
            TaskRequestCmd::RunTask(addr, param, task) => {
                let r = Self::do_run_task(&addr, &param, &client, &xxl_request_header).await;
                drop(permit);
                Ok((r, Some(task)))
            }
            TaskRequestCmd::RunBroadcastTask(addrs, param) => {
                let mut r = Ok(());
                for addr in addrs.iter() {
                    let t = Self::do_run_task(addr, &param, &client, &xxl_request_header).await;
                    if t.is_err() {
                        r = t;
                    }
                }
                drop(permit);
                Ok((r, None))
            }
        }
    }

    async fn do_run_task(
        instance_addr: &Arc<String>,
        param: &JobRunParam,
        client: &reqwest::Client,
        xxl_request_header: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let xxl_client = XxlClient::new(&client, &xxl_request_header, instance_addr);
        xxl_client.run_job(param).await?;
        Ok(())
    }
}

impl Actor for TaskRequestActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("TaskRequestActor started");
    }
}

impl Inject for TaskRequestActor {
    type Context = Context<Self>;
    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.batch_call_manager = factory_data.get_actor();
    }
}

impl Handler<TaskRequestCmd> for TaskRequestActor {
    type Result = ResponseActFuture<Self, anyhow::Result<TaskRequestResult>>;

    fn handle(&mut self, msg: TaskRequestCmd, _ctx: &mut Context<Self>) -> Self::Result {
        self.running_count += 1;
        let client = self.client.clone();
        let xxl_request_header = self.xxl_request_header.clone();
        let semaphore = self.request_semaphore.clone();
        let fut = Self::async_run_task(msg, xxl_request_header, client, semaphore)
            .into_actor(self)
            .map(|res, act, _ctx| {
                act.running_count -= 1;
                match res {
                    Ok((r, task_info)) => {
                        if let Some(mut task_info) = task_info {
                            match r {
                                Ok(_) => {
                                    task_info.status = TaskStatusType::Running;
                                }
                                Err(err) => {
                                    log::error!("run task error:{}", &err);
                                    task_info.status = TaskStatusType::Fail;
                                    task_info.trigger_message = Arc::new(err.to_string());
                                    task_info.finish_time = now_second_u32();
                                }
                            };
                            if let Some(raft_request_route) = act.batch_call_manager.as_ref() {
                                raft_request_route.do_send(BatchUpdateTaskManagerReq::UpdateTask(
                                    Arc::new(task_info),
                                ));
                            }
                        }
                        Ok(TaskRequestResult::RunningCount(act.running_count))
                    }
                    Err(err) => Err(err),
                }
            });
        Box::pin(fut)
    }
}
