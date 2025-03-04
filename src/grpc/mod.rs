use async_trait::async_trait;
use std::sync::Arc;

pub use crate::grpc::payload_utils::PayloadUtils;
use ratch_server_proto::Payload;

pub mod handler;
pub mod payload_utils;
pub mod ratch_server_proto;
pub mod server;

#[derive(Default)]
pub struct RequestMeta {
    pub connection_id: Arc<String>,
}

pub struct HandlerResult {
    pub success: bool,
    pub payload: Payload,
    pub message: Option<String>,
}

impl HandlerResult {
    pub fn success(payload: Payload) -> Self {
        Self {
            success: true,
            message: None,
            payload,
        }
    }

    pub fn error(code: u16, message: String) -> Self {
        let payload = PayloadUtils::build_error_payload(code, message.clone());
        Self {
            success: false,
            message: Some(message),
            payload,
        }
    }

    pub fn error_mark(payload: Payload) -> Self {
        Self {
            success: false,
            message: None,
            payload,
        }
    }

    pub fn error_with_message(payload: Payload, message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            payload,
        }
    }
}

#[async_trait]
pub trait PayloadHandler {
    async fn handle(
        &self,
        request_payload: Payload,
        request_meta: RequestMeta,
    ) -> anyhow::Result<HandlerResult>;
}
