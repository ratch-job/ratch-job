use crate::common::http_utils::{HttpUtils, ResponseWrap};
use crate::openapi::xxljob::model::XxlApiResult;
use crate::task::model::request_model::{JobLogInfo, JobLogParam, JobRunParam};
use serde::de::DeserializeOwned;
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
        let result: XxlApiResult<String> = self.request(body, "run", None).await?;
        if result.is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "call response error,url:{},msg:{}",
                self.build_url("run"),
                result.msg.unwrap_or_default()
            ))
        }
    }

    pub async fn read_log(
        &self,
        param: &JobLogParam,
        max_body_bytes: usize,
    ) -> anyhow::Result<JobLogInfo> {
        let body = serde_json::to_vec(param)?;
        let result: XxlApiResult<JobLogInfo> =
            self.request(body, "log", Some(max_body_bytes)).await?;
        if !result.is_success() {
            return Err(anyhow::anyhow!(
                "call response error,url:{},msg:{}",
                self.build_url("log"),
                result.msg.unwrap_or_default()
            ));
        }
        result
            .content
            .ok_or_else(|| anyhow::anyhow!("executor log response content is empty"))
    }

    fn build_url(&self, sub_url: &str) -> String {
        if self.is_addr_end_bias {
            format!("{}{}", self.addr, &sub_url)
        } else {
            format!("{}/{}", self.addr, &sub_url)
        }
    }

    async fn request<T: DeserializeOwned>(
        &self,
        body: Vec<u8>,
        sub_url: &str,
        max_body_bytes: Option<usize>,
    ) -> anyhow::Result<XxlApiResult<T>> {
        let url = self.build_url(sub_url);
        let response = if let Some(max_body_bytes) = max_body_bytes {
            HttpUtils::request_with_max_body(
                self.client,
                "POST",
                &url,
                body,
                Some(self.headers),
                Some(3000),
                max_body_bytes,
            )
            .await?
        } else {
            HttpUtils::request(
                self.client,
                "POST",
                &url,
                body,
                Some(self.headers),
                Some(3000),
            )
            .await?
        };
        Self::convert(&response).map_err(|error| {
            anyhow::anyhow!("parse executor response error,url:{},error:{}", url, error)
        })
    }

    fn convert<T: DeserializeOwned>(resp: &ResponseWrap) -> anyhow::Result<XxlApiResult<T>> {
        let v = serde_json::from_slice(&resp.body)?;
        Ok(v)
    }
}
