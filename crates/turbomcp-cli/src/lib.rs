//! # `TurboMCP` CLI
//!
//! Command-line interface for interacting with MCP servers, providing tools for
//! testing, debugging, and managing MCP server instances.
//!
//! ## Features
//!
//! - Connect to MCP servers via multiple transports (HTTP, WebSocket, STDIO)
//! - List available tools and their schemas
//! - Call tools with JSON arguments
//! - Export tool schemas for documentation
//! - Support for authentication via bearer tokens
//! - JSON and human-readable output formats
//!
//! ## Usage
//!
//! ```bash
//! # List tools from HTTP server
//! turbomcp-cli tools-list --transport http --url http://localhost:8080/mcp
//!
//! # Call a tool with arguments
//! turbomcp-cli tools-call --transport http --url http://localhost:8080/mcp \
//!   add --arguments '{"a": 5, "b": 3}'
//!
//! # Export tool schemas
//! turbomcp-cli schema-export --transport http --url http://localhost:8080/mcp --json
//! ```

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// Main CLI application structure
#[derive(Parser, Debug)]
#[command(
    name = "turbomcp-cli",
    version,
    about = "TurboMCP command-line interface"
)]
pub struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List tools from a running server
    ToolsList(Connection),
    /// Call a tool on a running server
    ToolsCall {
        #[command(flatten)]
        conn: Connection,
        /// Tool name
        name: String,
        /// Arguments as JSON (object)
        #[arg(long, default_value = "{}")]
        arguments: String,
    },
    /// Export tool schemas from a running server
    SchemaExport(Connection),
}

/// Run the CLI application
pub fn run_cli() {
    let cli = Cli::parse();
    let rt = Runtime::new().expect("tokio rt");
    rt.block_on(async move {
        match cli.command {
            Commands::ToolsList(conn) => {
                if let Err(e) = cmd_tools_list(conn).await {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
            Commands::ToolsCall {
                conn,
                name,
                arguments,
            } => {
                if let Err(e) = cmd_tools_call(conn, name, arguments).await {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
            Commands::SchemaExport(conn) => {
                if let Err(e) = cmd_schema_export(conn).await {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
    });
}

/// Connection configuration for connecting to MCP servers
#[derive(Args, Debug, Clone)]
pub struct Connection {
    /// Transport: stdio | http | ws
    #[arg(long, value_enum, default_value_t = TransportKind::Stdio)]
    pub transport: TransportKind,
    /// Server URL for http/ws (ignored for stdio)
    #[arg(long, default_value = "http://localhost:8080/mcp")]
    pub url: String,
    /// Bearer token or API key
    #[arg(long)]
    pub auth: Option<String>,
    /// Emit JSON output
    #[arg(long)]
    pub json: bool,
}

/// Available transport types for connecting to MCP servers
#[derive(Debug, Clone, ValueEnum)]
pub enum TransportKind {
    /// Standard input/output transport
    Stdio,
    /// HTTP transport with JSON-RPC
    Http,
    /// WebSocket transport
    Ws,
}

pub async fn cmd_tools_list(conn: Connection) -> Result<(), String> {
    match conn.transport {
        TransportKind::Http => http_list_tools(&conn).await,
        TransportKind::Ws => ws_list_tools(&conn).await,
        TransportKind::Stdio => stdio_list_tools(&conn).await,
    }
}

pub async fn cmd_tools_call(
    conn: Connection,
    name: String,
    arguments: String,
) -> Result<(), String> {
    match conn.transport {
        TransportKind::Http => http_call_tool(&conn, name, arguments).await,
        TransportKind::Ws => ws_call_tool(&conn, name, arguments).await,
        TransportKind::Stdio => stdio_call_tool(&conn, name, arguments).await,
    }
}

pub async fn cmd_schema_export(conn: Connection) -> Result<(), String> {
    match conn.transport {
        TransportKind::Http => http_schema_export(&conn).await,
        TransportKind::Ws => ws_schema_export(&conn).await,
        TransportKind::Stdio => stdio_schema_export(&conn).await,
    }
}

async fn http_list_tools(conn: &Connection) -> Result<(), String> {
    let req = json!({"jsonrpc":"2.0","id":"1","method":"tools/list"});
    let res = http_post(conn, req).await?;
    output(conn, &res)
}

async fn http_call_tool(conn: &Connection, name: String, arguments: String) -> Result<(), String> {
    let args_map: HashMap<String, serde_json::Value> =
        serde_json::from_str(&arguments).map_err(|e| format!("invalid --arguments JSON: {e}"))?;
    let req = json!({
        "jsonrpc":"2.0","id":"1","method":"tools/call",
        "params": {"name": name, "arguments": args_map}
    });
    let res = http_post(conn, req).await?;
    output(conn, &res)
}

async fn http_schema_export(conn: &Connection) -> Result<(), String> {
    // List, then print each tool's inputSchema
    let req = json!({"jsonrpc":"2.0","id":"1","method":"tools/list"});
    let res = http_post(conn, req).await?;
    if let Some(result) = res.get("result")
        && let Some(tools) = result.get("tools").and_then(|v| v.as_array())
    {
        let mut out = vec![];
        for t in tools {
            let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let schema = t.get("inputSchema").cloned().unwrap_or(json!({}));
            out.push(json!({"name": name, "schema": schema}));
        }
        return output(conn, &json!({"schemas": out}));
    }
    output(conn, &res)
}

async fn http_post(
    conn: &Connection,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let mut req = client.post(&conn.url).json(&body);
    if let Some(auth) = &conn.auth {
        req = req.bearer_auth(auth);
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {status}: {text}"));
    }
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON: {e}"))
}

// WebSocket implementation functions
async fn ws_list_tools(conn: &Connection) -> Result<(), String> {
    use serde_json::json;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let response = ws_send_request(conn, request).await?;
    output(conn, &response)
}

async fn ws_call_tool(conn: &Connection, name: String, arguments: String) -> Result<(), String> {
    use serde_json::json;

    let args: serde_json::Value =
        serde_json::from_str(&arguments).map_err(|e| format!("Invalid JSON arguments: {e}"))?;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": args
        }
    });

    let response = ws_send_request(conn, request).await?;
    output(conn, &response)
}

async fn ws_schema_export(conn: &Connection) -> Result<(), String> {
    use serde_json::json;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list",
        "params": {}
    });

    let response = ws_send_request(conn, request).await?;

    // Transform response to extract schemas
    if let Some(result) = response.get("result")
        && let Some(tools) = result.get("tools").and_then(|t| t.as_array())
    {
        let mut out = Vec::new();
        for tool in tools {
            let name = tool
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");
            let schema = tool.get("inputSchema").cloned().unwrap_or(json!({}));
            out.push(json!({"name": name, "schema": schema}));
        }
        return output(conn, &json!({"schemas": out}));
    }
    output(conn, &response)
}

async fn ws_send_request(
    conn: &Connection,
    request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Convert HTTP/HTTPS URL to WebSocket URL
    let ws_url = conn
        .url
        .replace("http://", "ws://")
        .replace("https://", "wss://")
        .replace("/mcp", "/ws");

    // Connect to WebSocket server
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("Failed to connect to WebSocket at {ws_url}: {e}"))?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send the JSON-RPC request
    let request_text =
        serde_json::to_string(&request).map_err(|e| format!("Failed to serialize request: {e}"))?;

    ws_sender
        .send(Message::Text(request_text))
        .await
        .map_err(|e| format!("Failed to send WebSocket message: {e}"))?;

    // Wait for response
    match ws_receiver.next().await {
        Some(Ok(Message::Text(response_text))) => serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse JSON response: {e}")),
        Some(Ok(msg)) => Err(format!("Unexpected WebSocket message type: {msg:?}")),
        Some(Err(e)) => Err(format!("WebSocket error: {e}")),
        None => Err("WebSocket connection closed unexpectedly".to_string()),
    }
}

// Stdio implementation functions
async fn stdio_list_tools(conn: &Connection) -> Result<(), String> {
    use serde_json::json;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let response = stdio_send_request(conn, request).await?;
    output(conn, &response)
}

async fn stdio_call_tool(conn: &Connection, name: String, arguments: String) -> Result<(), String> {
    use serde_json::json;

    let args: serde_json::Value =
        serde_json::from_str(&arguments).map_err(|e| format!("Invalid JSON arguments: {e}"))?;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": args
        }
    });

    let response = stdio_send_request(conn, request).await?;
    output(conn, &response)
}

async fn stdio_schema_export(conn: &Connection) -> Result<(), String> {
    use serde_json::json;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list",
        "params": {}
    });

    let response = stdio_send_request(conn, request).await?;

    // Transform response to extract schemas
    if let Some(result) = response.get("result")
        && let Some(tools) = result.get("tools").and_then(|t| t.as_array())
    {
        let mut out = Vec::new();
        for tool in tools {
            let name = tool
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");
            let schema = tool.get("inputSchema").cloned().unwrap_or(json!({}));
            out.push(json!({"name": name, "schema": schema}));
        }
        return output(conn, &json!({"schemas": out}));
    }
    output(conn, &response)
}

async fn stdio_send_request(
    conn: &Connection,
    request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    // Execute the command specified in conn.url as a STDIO MCP server
    let mut parts = conn.url.split_whitespace();
    let command = parts
        .next()
        .ok_or("No command specified for STDIO transport")?;
    let args: Vec<&str> = parts.collect();

    let mut child = Command::new(command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command '{command}': {e}"))?;

    // Send request
    let stdin = child.stdin.as_mut().ok_or("Failed to get stdin handle")?;
    let request_str =
        serde_json::to_string(&request).map_err(|e| format!("Failed to serialize request: {e}"))?;
    writeln!(stdin, "{request_str}").map_err(|e| format!("Failed to write request: {e}"))?;

    // Read response
    let stdout = child.stdout.take().ok_or("Failed to get stdout handle")?;
    let mut reader = BufReader::new(stdout);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("Failed to read response: {e}"))?;

    // Wait for process to complete
    let output = child
        .wait()
        .map_err(|e| format!("Process execution failed: {e}"))?;

    if !output.success() {
        return Err(format!(
            "Command failed with exit code: {}",
            output.code().unwrap_or(-1)
        ));
    }

    // Parse JSON response
    serde_json::from_str(&response_line).map_err(|e| format!("Invalid JSON response: {e}"))
}

pub fn output(conn: &Connection, value: &serde_json::Value) -> Result<(), String> {
    if conn.json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
        );
    } else {
        println!("{value}");
    }
    Ok(())
}
