use crate::common::share_data::ShareData;
use crate::grpc::handler::InvokerHandler;
use crate::grpc::ratch_server_proto::{request_server, Payload, Response};
use crate::grpc::{PayloadHandler, RequestMeta};
use std::sync::Arc;
use tonic::{Request, Status};

pub struct RequestServerImpl {
    pub(crate) share_data: Arc<ShareData>,
    invoker: InvokerHandler,
}

impl RequestServerImpl {
    pub fn new(share_data: Arc<ShareData>, invoker: InvokerHandler) -> Self {
        RequestServerImpl {
            share_data,
            invoker,
        }
    }
}

#[tonic::async_trait]
impl request_server::Request for RequestServerImpl {
    async fn request(
        &self,
        request: Request<Payload>,
    ) -> Result<tonic::Response<Response>, Status> {
        let remote_addr = request.remote_addr().unwrap();
        let payload = request.into_inner();
        let meta = RequestMeta {
            connection_id: Arc::new(remote_addr.to_string()),
        };
        let result = self.invoker.handle(payload, meta).await;
        match result {
            Ok(result) => Ok(tonic::Response::new(Response {
                code: 0,
                data: Some(result.payload),
                message: "".to_string(),
            })),
            Err(err) => Ok(tonic::Response::new(Response {
                code: 500,
                message: err.to_string(),
                data: None,
            })),
        }
    }
}
