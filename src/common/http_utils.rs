use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Body, Error, IntoUrl, Response, Url};
use reqwest::header::HeaderMap;

#[derive(Default, Clone, Debug)]
pub struct ResponseWrap {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl ResponseWrap {
    pub fn status_is_200(&self) -> bool {
        self.status == 200
    }

    pub fn get_lossy_string_body(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.body)
    }

    pub fn get_string_body(&self) -> String {
        String::from_utf8(self.body.clone()).unwrap()
    }

    pub fn get_map_headers(&self) -> HashMap<String, String> {
        Self::convert_to_map_headers(self.headers.clone())
    }

    pub fn convert_to_map_headers(headers: Vec<(String, String)>) -> HashMap<String, String> {
        let mut h = HashMap::new();
        for (k, v) in headers {
            h.insert(k, v);
        }
        h
    }
}

pub struct HttpUtils;

impl HttpUtils {
    async fn get_response_wrap(resp: reqwest::Response) -> anyhow::Result<ResponseWrap> {
        let status = resp.status().as_u16();
        let mut resp_headers = vec![];
        for (k, v) in resp.headers() {
            let value = String::from_utf8(v.as_bytes().to_vec())?;
            resp_headers.push((k.as_str().to_owned(), value));
        }
        let body = resp.bytes().await?.to_vec();
        Ok(ResponseWrap {
            status,
            headers: resp_headers,
            body,
        })
    }

    pub async fn request(
        client: &reqwest::Client,
        method_name: &str,
        url: &str,
        body: Vec<u8>,
        headers: Option<&HashMap<String, String>>,
        timeout_millis: Option<u64>,
    ) -> anyhow::Result<ResponseWrap> {
        let mut req_builer = match method_name {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => client.post(url),
        };
        if let Some(headers) = headers {
            for (k, v) in headers.iter() {
                req_builer = req_builer.header(k, v.to_string());
            }
        }
        if let Some(timeout) = timeout_millis {
            req_builer = req_builer.timeout(Duration::from_millis(timeout));
        }
        if !body.is_empty() {
            req_builer = req_builer.body(body);
        }
        let res = req_builer.send().await?;
        Self::get_response_wrap(res).await
    }

    pub async fn post_body<T: IntoUrl, B: Into<Body>>(url: T, body: B, headers: Option<HeaderMap>, timeout_millis: Option<u64>) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        client.post(url)
            .body(body)
            .headers(headers.unwrap_or(HeaderMap::default()))
            .timeout(Duration::from_millis(timeout_millis.unwrap_or(3000u64)))
            .send()
            .await?;
        Ok(())
    }
}
