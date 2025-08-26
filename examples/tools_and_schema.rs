//! Tools and JSON handling example (no macros)

use serde::{Deserialize, Serialize};
use turbomcp_core::RequestContext;
use turbomcp_protocol::types::{CallToolRequest, CallToolResult, Content, TextContent};
use turbomcp_server::ServerError;
use turbomcp_server::{handlers::utils, ServerBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddParams {
    a: f64,
    b: f64,
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    tracing_subscriber::fmt::init();

    let mut builder = ServerBuilder::new().name("SchemaDemo").version("1.0.0");

    // add tool with typed params parsing
    builder = builder.tool(
        "add",
        utils::tool(
            "add",
            "Add two numbers",
            |req: CallToolRequest, _ctx: RequestContext| async move {
                let params = req
                    .arguments
                    .as_ref()
                    .and_then(|m| {
                        serde_json::from_value::<AddParams>(serde_json::Value::Object(
                            m.clone().into_iter().collect(),
                        ))
                        .ok()
                    })
                    .unwrap_or(AddParams { a: 0.0, b: 0.0 });
                Ok(CallToolResult {
                    content: vec![Content::Text(TextContent {
                        text: (params.a + params.b).to_string(),
                        annotations: None,
                        meta: None,
                    })],
                    is_error: None,
                })
            },
        ),
    )?;

    // echo tool using raw JSON
    builder = builder.tool(
        "echo",
        utils::tool(
            "echo",
            "Echo a message",
            |req: CallToolRequest, _ctx: RequestContext| async move {
                let msg = req
                    .arguments
                    .as_ref()
                    .and_then(|m| m.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(CallToolResult {
                    content: vec![Content::Text(TextContent {
                        text: msg,
                        annotations: None,
                        meta: None,
                    })],
                    is_error: None,
                })
            },
        ),
    )?;

    let server = builder.build();
    Ok(server.run_stdio().await?)
}
