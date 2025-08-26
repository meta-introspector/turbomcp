//! Hello World TurboMCP server (modern, no macros)

use turbomcp_core::RequestContext;
use turbomcp_protocol::types::{CallToolRequest, CallToolResult, Content, TextContent};
use turbomcp_server::ServerError;
use turbomcp_server::{handlers::utils, ServerBuilder};

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    tracing_subscriber::fmt::init();

    // Build a server with two simple tools
    let mut builder = ServerBuilder::new().name("HelloWorld").version("1.0.0");

    // hello tool
    builder = builder.tool(
        "hello",
        utils::tool(
            "hello",
            "Say hello",
            |req: CallToolRequest, _ctx: RequestContext| async move {
                let name = req
                    .arguments
                    .as_ref()
                    .and_then(|m| m.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("world")
                    .to_string();
                Ok(CallToolResult {
                    content: vec![Content::Text(TextContent {
                        text: format!("Hello, {}!", name),
                        annotations: None,
                        meta: None,
                    })],
                    is_error: None,
                })
            },
        ),
    )?;

    // health tool
    builder = builder.tool(
        "health",
        utils::tool(
            "health",
            "Basic health check",
            |_req: CallToolRequest, _ctx: RequestContext| async move {
                Ok(CallToolResult {
                    content: vec![Content::Text(TextContent {
                        text: "ok".to_string(),
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
