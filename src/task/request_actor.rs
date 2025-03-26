use crate::common::app_config::AppConfig;
use crate::common::datetime_utils::now_second_u32;
use crate::common::get_app_version;
use crate::job::model::actor_model::JobManagerRaftReq;
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::ClientRequest;
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
    raft_request_route: Option<Arc<RaftRequestRoute>>,
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
            raft_request_route: None,
        }
    }

    fn run_task(
        &mut self,
        addr: Arc<String>,
        param: JobRunParam,
        mut task_info: JobTaskInfo,
        ctx: &mut Context<Self>,
    ) {
        //let raft_request_route = self.raft_request_route.clone();
        let client = self.client.clone();
        let xxl_request_header = self.xxl_request_header.clone();
        let raft_request_route = self.raft_request_route.clone();
        async move {
            let result = Self::do_run_task(addr, &param, &client, &xxl_request_header).await;
            if let Err(err) = result {
                log::error!("run task error:{}", &err);
                if let Some(raft_request_route) = raft_request_route {
                    task_info.status = TaskStatusType::Fail;
                    task_info.trigger_message = Arc::new(err.to_string());
                    task_info.finish_time = now_second_u32();
                    return Self::notify_update_task(
                        &raft_request_route,
                        vec![Arc::new(task_info)],
                    )
                    .await;
                }
            }
            Ok(())
        }
        .into_actor(self)
        .map(|result, _, _| {})
        .spawn(ctx);
    }

    fn run_broadcast_task(
        &mut self,
        addrs: Vec<Arc<String>>,
        param: JobRunParam,
        ctx: &mut Context<Self>,
    ) {
        let client = self.client.clone();
        let xxl_request_header = self.xxl_request_header.clone();
        async move {
            for addr in addrs {
                Self::do_run_task(addr, &param, &client, &xxl_request_header)
                    .await
                    .ok();
            }
            Ok(())
        }
        .into_actor(self)
        .map(|_r: anyhow::Result<()>, _, _| {})
        .spawn(ctx);
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
        self.raft_request_route = factory_data.get_bean();
    }
}

impl Handler<TaskRequestCmd> for TaskRequestActor {
    type Result = anyhow::Result<TaskRequestResult>;

    fn handle(&mut self, msg: TaskRequestCmd, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            TaskRequestCmd::RunTask(addr, params, task) => {
                self.run_task(addr, params, task, ctx);
            }
            TaskRequestCmd::RunBroadcastTask(addrs, params) => {
                self.run_broadcast_task(addrs, params, ctx);
            }
        }
        Ok(TaskRequestResult::None)
    }
}
