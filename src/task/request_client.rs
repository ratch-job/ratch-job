use crate::common::http_utils::{HttpUtils, ResponseWrap};
use crate::openapi::xxljob::model::XxlApiResult;
use crate::task::model::request_model::JobRunParam;
use std::collections::HashMap;
use std::sync::Arc;

pub struct XxlClient<'a> {
    pub client: &'a reqwest::Client,
    pub headers: &'a HashMap<String, String>,
    pub addr: &'a Arc<String>,
    is_addr_end_bias: bool,
}

impl<'a> XxlClient<'a> {
    pub fn new(
        client: &'a reqwest::Client,
        headers: &'a HashMap<String, String>,
        addr: &'a Arc<String>,
    ) -> Self {
        let is_addr_end_bias = addr.ends_with("/");
        XxlClient {
            client,
            headers,
            addr,
            is_addr_end_bias,
        }
    }

    pub async fn run_job(&self, param: &JobRunParam) -> anyhow::Result<()> {
        let body = serde_json::to_vec(param)?;
        match self.request(body, "run").await {
            Ok(_) => {
                //log::info!("XxlClient|run success");
                Ok(())
            }
            Err(e) => {
                log::error!("XxlClient|run error:{}", &e);
                Err(e)
            }
        }
    }

    async fn request(&self, body: Vec<u8>, sub_url: &str) -> anyhow::Result<()> {
        let mut registry_success = false;
        let url = if self.is_addr_end_bias {
            format!("{}{}", self.addr, &sub_url)
        } else {
            format!("{}/{}", self.addr, &sub_url)
        };
        match HttpUtils::request(
            &self.client,
            "POST",
            &url,
            body.clone(),
            Some(&self.headers),
            Some(3000),
        )
        .await
        {
            Ok(resp) => {
                if let Ok(v) = Self::convert(&resp) {
                    if v.is_success() {
                        registry_success = true;
                    }
                }
                if !registry_success {
                    log::error!(
                        "call response error:,url:{},resp:{}",
                        &url,
                        resp.get_lossy_string_body()
                    );
                }
            }
            Err(err) => {
                log::error!("call response error:{},url:{}", err, &url);
            }
        }
        if !registry_success {
            Err(anyhow::anyhow!(
                "call failed,url:{},body:{}",
                &url,
                String::from_utf8_lossy(&body)
            ))
        } else {
            Ok(())
        }
    }

    fn convert(resp: &ResponseWrap) -> anyhow::Result<XxlApiResult<String>> {
        let v = serde_json::from_slice(&resp.body)?;
        Ok(v)
    }
}
