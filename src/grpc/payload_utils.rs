use crate::grpc::ratch_server_proto::{Any, Payload};
use std::collections::HashMap;

pub struct PayloadUtils;

impl PayloadUtils {
    pub fn build_error_payload(error_code: u16, error_msg: String) -> Payload {
        //let error_val = BaseResponse::build_error_response(error_code, error_msg).to_json_string();
        let error_val = error_msg.into_bytes();
        Self::build_payload("ErrorResponse", error_val)
    }

    pub fn build_empty_payload() -> Payload {
        Payload {
            r#type: "".into(),
            headers: Default::default(),
            body: None,
        }
    }

    pub fn build_payload(r#type: &str, val: Vec<u8>) -> Payload {
        Self::build_full_payload(r#type, val, Default::default())
    }

    pub fn build_full_payload(
        r#type: &str,
        val: Vec<u8>,
        headers: HashMap<String, String>,
    ) -> Payload {
        let body = Any {
            type_url: "".into(),
            value: val,
        };
        Payload {
            r#type: r#type.to_string(),
            headers,
            body: Some(body),
        }
    }

    pub fn get_payload_header(payload: &Payload) -> String {
        let mut str = String::default();
        str.push_str(&format!("type:{},", payload.r#type));
        str.push_str(&format!("header:{:?},", &payload.headers));
        str
    }

    pub fn get_payload_string(payload: &Payload) -> String {
        let mut str = String::default();
        str.push_str(&format!("type:{},", payload.r#type));
        str.push_str(&format!("header:{:?},", &payload.headers));
        if let Some(body) = &payload.body {
            let value_str = String::from_utf8_lossy(&body.value);
            str.push_str(&format!("body:{}", value_str));
        }
        str
    }
}
