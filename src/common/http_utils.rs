use futures_util::StreamExt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

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

    pub fn get_lossy_string_body(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    pub fn get_string_body(&self) -> String {
        String::from_utf8(self.body.clone())
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
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
    async fn get_response_wrap(
        resp: reqwest::Response,
        max_body_bytes: Option<usize>,
    ) -> anyhow::Result<ResponseWrap> {
        let status = resp.status().as_u16();
        let mut resp_headers = vec![];
        for (k, v) in resp.headers() {
            let value = String::from_utf8(v.as_bytes().to_vec())?;
            resp_headers.push((k.as_str().to_owned(), value));
        }
        if let (Some(max_body_bytes), Some(content_length)) =
            (max_body_bytes, resp.content_length())
        {
            if content_length > max_body_bytes as u64 {
                return Err(anyhow::anyhow!(
                    "response body exceeds limit: {} bytes",
                    max_body_bytes
                ));
            }
        }
        let mut body = Vec::new();
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if let Some(max_body_bytes) = max_body_bytes {
                if body.len() + chunk.len() > max_body_bytes {
                    return Err(anyhow::anyhow!(
                        "response body exceeds limit: {} bytes",
                        max_body_bytes
                    ));
                }
            }
            body.extend_from_slice(&chunk);
        }
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
        Self::request_inner(
            client,
            method_name,
            url,
            body,
            headers,
            timeout_millis,
            None,
        )
        .await
    }

    pub async fn request_with_max_body(
        client: &reqwest::Client,
        method_name: &str,
        url: &str,
        body: Vec<u8>,
        headers: Option<&HashMap<String, String>>,
        timeout_millis: Option<u64>,
        max_body_bytes: usize,
    ) -> anyhow::Result<ResponseWrap> {
        Self::request_inner(
            client,
            method_name,
            url,
            body,
            headers,
            timeout_millis,
            Some(max_body_bytes),
        )
        .await
    }

    async fn request_inner(
        client: &reqwest::Client,
        method_name: &str,
        url: &str,
        body: Vec<u8>,
        headers: Option<&HashMap<String, String>>,
        timeout_millis: Option<u64>,
        max_body_bytes: Option<usize>,
    ) -> anyhow::Result<ResponseWrap> {
        let mut req_builder = match method_name {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => client.post(url),
        };
        if let Some(headers) = headers {
            for (key, value) in headers {
                req_builder = req_builder.header(key, value);
            }
        }
        if let Some(timeout) = timeout_millis {
            req_builder = req_builder.timeout(Duration::from_millis(timeout));
        }
        if !body.is_empty() {
            req_builder = req_builder.body(body);
        }
        let response = req_builder.send().await?;
        Self::get_response_wrap(response, max_body_bytes).await
    }
}
