use std::sync::Arc;

use crate::common::app_config::AppConfig;
use crate::grpc::handler::CLUSTER_TOKEN;
use crate::grpc::ratch_server_proto;
use crate::grpc::ratch_server_proto::request_client::RequestClient;
use crate::grpc::ratch_server_proto::Payload;
use actix::prelude::*;
use inner_mem_cache::MemCache;
use tonic::transport::Channel;

#[derive(Debug)]
pub struct RaftClusterRequestSender {
    conn_factory: Addr<RaftConnectionFactory>,
    app_config: Arc<AppConfig>,
}

impl RaftClusterRequestSender {
    pub fn new(conn_factory: Addr<RaftConnectionFactory>, sys_config: Arc<AppConfig>) -> Self {
        Self {
            conn_factory,
            app_config: sys_config,
        }
    }

    pub async fn get_node_channel(&self, addr: Arc<String>) -> anyhow::Result<Arc<Channel>> {
        let res: RaftConnResponse = self
            .conn_factory
            .send(RaftConnRequest::GetChannel(addr))
            .await??;
        match res {
            RaftConnResponse::Channel(channel) => Ok(channel),
            RaftConnResponse::None => Err(anyhow::anyhow!("get raft conn error")),
        }
    }

    pub async fn send_request(
        &self,
        addr: Arc<String>,
        mut payload: Payload,
    ) -> anyhow::Result<Payload> {
        let channel = self.get_node_channel(addr.clone()).await?;
        let mut request_client = RequestClient::new(channel.as_ref().clone());
        if !self.app_config.cluster_token.is_empty() {
            payload.headers.insert(
                CLUSTER_TOKEN.to_string(),
                self.app_config.cluster_token.as_str().to_string(),
            );
        }
        let resp = match request_client.request(payload).await {
            Ok(resp) => {
                self.conn_factory.do_send(RaftConnRequest::UpdateChannel {
                    key: addr,
                    is_active: true,
                });
                resp
            }
            Err(err) => {
                self.conn_factory.do_send(RaftConnRequest::UpdateChannel {
                    key: addr,
                    is_active: false,
                });
                return Err(err.into());
            }
        };
        let response: ratch_server_proto::Response = resp.into_inner();
        if response.code != 0 {
            return Err(anyhow::anyhow!(
                "raft target response error,{}",
                response.message
            ));
        }
        if let Some(payload) = response.data {
            Ok(payload)
        } else {
            Err(anyhow::anyhow!("raft target response error"))
        }
    }
}

pub struct RaftConnectionFactory {
    channel_cache: MemCache<Arc<String>, Arc<Channel>>,
    cache_ses: i32,
}

impl RaftConnectionFactory {
    pub fn new(cache_ses: i32) -> Self {
        Self {
            channel_cache: MemCache::<Arc<String>, Arc<Channel>>::new(),
            cache_ses,
        }
    }

    fn build_channel(&mut self, key: Arc<String>) -> anyhow::Result<Arc<Channel>> {
        self.channel_cache.clear_time_out();
        if let Ok(channel) = self.channel_cache.get(&key) {
            Ok(channel)
        } else {
            let addr = format!("http://{}", &key);
            let channel = Arc::new(Channel::from_shared(addr)?.connect_lazy());
            self.channel_cache.set(key, channel.clone(), self.cache_ses);
            Ok(channel)
        }
    }

    fn update_channel_status(&mut self, key: Arc<String>, is_active: bool) {
        if let Ok(channel) = self.channel_cache.get(&key) {
            if is_active {
                let tll = self.channel_cache.time_to_live(&key);
                if tll < 10 {
                    self.channel_cache.set(key, channel, self.cache_ses);
                }
            } else {
                self.channel_cache.remove(&key);
            }
        }
    }
}

impl Actor for RaftConnectionFactory {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("ConnectionFactory started")
    }
}

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<RaftConnResponse>")]
pub enum RaftConnRequest {
    GetChannel(Arc<String>),
    UpdateChannel { key: Arc<String>, is_active: bool },
}

pub enum RaftConnResponse {
    Channel(Arc<Channel>),
    None,
}

impl Handler<RaftConnRequest> for RaftConnectionFactory {
    type Result = anyhow::Result<RaftConnResponse>;

    fn handle(&mut self, msg: RaftConnRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            RaftConnRequest::GetChannel(key) => {
                let c = self.build_channel(key)?;
                Ok(RaftConnResponse::Channel(c))
            }
            RaftConnRequest::UpdateChannel { key, is_active } => {
                self.update_channel_status(key, is_active);
                Ok(RaftConnResponse::None)
            }
        }
    }
}
