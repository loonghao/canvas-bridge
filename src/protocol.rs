use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CanvasCommand {
    #[serde(rename = "canvas.tools.search")]
    SearchTools {
        query: String,
        #[serde(default)]
        dcc_type: Option<String>,
        #[serde(default)]
        limit: Option<u32>,
    },
    #[serde(rename = "canvas.tool.describe")]
    DescribeTool { tool_slug: String },
    #[serde(rename = "canvas.node.run")]
    RunNode {
        run_id: String,
        node_id: String,
        tool_slug: String,
        #[serde(default)]
        arguments: Value,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum CanvasEvent {
    #[serde(rename = "canvas.bridge.ready")]
    Ready { gateway: String },
    #[serde(rename = "canvas.tools.search.result")]
    SearchResult { result: Value },
    #[serde(rename = "canvas.tool.describe.result")]
    DescribeResult { tool_slug: String, result: Value },
    #[serde(rename = "canvas.node.started")]
    NodeStarted {
        run_id: String,
        node_id: String,
        tool_slug: String,
    },
    #[serde(rename = "canvas.node.accepted")]
    NodeAccepted {
        run_id: String,
        node_id: String,
        job_id: Option<String>,
        result: Value,
    },
    #[serde(rename = "canvas.error")]
    Error { message: String },
}
