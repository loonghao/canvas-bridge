use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct GatewayClient {
    base: String,
    http: Client,
}

impl GatewayClient {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').to_string(),
            http: Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base
    }

    pub async fn search_tools(
        &self,
        query: &str,
        dcc_type: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Value> {
        let mut body = json!({ "query": query });
        if let Some(dcc_type) = dcc_type {
            body["dcc_type"] = json!(dcc_type);
        }
        if let Some(limit) = limit {
            body["limit"] = json!(limit);
        }
        self.post_json("/v1/search", body).await
    }

    pub async fn describe_tool(&self, tool_slug: &str) -> Result<Value> {
        self.post_json("/v1/describe", json!({ "tool_slug": tool_slug }))
            .await
    }

    pub async fn call_tool(
        &self,
        tool_slug: &str,
        arguments: Value,
        progress_token: &str,
    ) -> Result<Value> {
        self.post_json(
            "/v1/call",
            json!({
                "tool_slug": tool_slug,
                "arguments": object_or_empty(arguments),
                "meta": {
                    "dcc": { "async": true },
                    "progressToken": progress_token
                }
            }),
        )
        .await
    }

    async fn post_json(&self, path: &str, body: Value) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("failed to POST {url}"))?;
        let status = response.status();
        let value = response
            .json::<Value>()
            .await
            .with_context(|| format!("failed to decode JSON response from {url}"))?;
        if status.is_success() {
            Ok(value)
        } else {
            anyhow::bail!("gateway returned {status}: {value}");
        }
    }
}

pub fn extract_job_id(value: &Value) -> Option<String> {
    let candidates = [
        "/job_id",
        "/output/job_id",
        "/structuredContent/job_id",
        "/result/structuredContent/job_id",
        "/_meta/dcc/jobId",
        "/result/_meta/dcc/jobId",
    ];
    candidates
        .iter()
        .filter_map(|path| value.pointer(path))
        .find_map(|v| v.as_str().map(ToOwned::to_owned))
}

fn object_or_empty(value: Value) -> Value {
    if value.is_object() { value } else { json!({}) }
}
