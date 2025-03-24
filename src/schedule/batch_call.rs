use crate::openapi::xxljob::model::server_request::CallbackParam;
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::ClientRequest;
use crate::schedule::model::actor_model::ScheduleManagerRaftReq;
use crate::task::model::task::TaskCallBackParam;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use futures_util::task::SpawnExt;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;

#[derive(Debug, Default)]
pub struct CallbackGroup {
    pub params: Vec<TaskCallBackParam>,
    pub senders: Vec<Sender<bool>>,
}

impl CallbackGroup {
    pub fn new() -> Self {
        Self {
            params: vec![],
            senders: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

#[bean(inject)]
pub struct BatchCallManager {
    raft_request_route: Option<Arc<RaftRequestRoute>>,
    callback_cache: Option<CallbackGroup>,
}

impl BatchCallManager {
    pub fn new() -> Self {
        Self {
            raft_request_route: None,
            callback_cache: Some(CallbackGroup::new()),
        }
    }

    async fn do_callback(
        params: CallbackGroup,
        raft_request_route: Option<Arc<RaftRequestRoute>>,
    ) -> anyhow::Result<()> {
        let mut result = false;
        if let Some(raft_request_route) = raft_request_route {
            if let Ok(_) = raft_request_route
                .request(ClientRequest::ScheduleReq {
                    req: ScheduleManagerRaftReq::TaskCallBacks(params.params),
                })
                .await
            {
                result = true;
            }
        }
        for sender in params.senders {
            sender.send(result).ok();
        }
        Ok(())
    }

    fn cache_is_empty(&self) -> bool {
        if let Some(callback_cache) = self.callback_cache.as_ref() {
            callback_cache.is_empty()
        } else {
            true
        }
    }

    fn callback(&mut self, ctx: &mut Context<Self>) {
        if !self.cache_is_empty() {
            let old_group = self.callback_cache.replace(CallbackGroup::new());
            if old_group.is_none() {
                return;
            }
            let group = old_group.unwrap();
            let raft_request_route = self.raft_request_route.clone();
            BatchCallManager::do_callback(group, raft_request_route)
                .into_actor(self)
                .map(|_res, _act, _ctx| {})
                .spawn(ctx);
        }
    }

    fn heartbeat(&mut self, ctx: &mut Context<Self>) {
        ctx.run_later(std::time::Duration::from_millis(450), move |act, ctx| {
            act.callback(ctx);
            act.heartbeat(ctx);
        });
    }
}

impl Actor for BatchCallManager {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("BatchCallManager started")
    }
}

impl Inject for BatchCallManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        ctx: &mut Self::Context,
    ) {
        self.raft_request_route = factory_data.get_bean();
        self.heartbeat(ctx);
    }
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<()>")]
pub enum BatchCallManagerReq {
    Callback(Vec<CallbackParam>),
}

impl Handler<BatchCallManagerReq> for BatchCallManager {
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, msg: BatchCallManagerReq, _ctx: &mut Self::Context) -> Self::Result {
        let rx = match msg {
            BatchCallManagerReq::Callback(params) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                if let Some(callback_cache) = self.callback_cache.as_mut() {
                    for param in params {
                        callback_cache.params.push(param.into());
                    }
                    callback_cache.senders.push(tx);
                }
                rx
            }
        };
        let fut = async move {
            if rx.await? {
                Ok(())
            } else {
                Err(anyhow::anyhow!("callback error"))
            }
        }
        .into_actor(self)
        .map(|res: anyhow::Result<()>, _act, _ctx| res);
        Box::pin(fut)
    }
}
