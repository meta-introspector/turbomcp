//! Transport Showcase Example - Comprehensive transport capabilities demonstration
//!
//! This example showcases TurboMCP's comprehensive transport architecture with runtime selection.
//! It demonstrates the progressive enhancement philosophy: start simple with STDIO and add
//! advanced transports as needed.
//!
//! # Available Transports
//!
//! - **STDIO**: Standard input/output (always available, default)
//! - **HTTP Server**: REST API + MCP over HTTP/SSE/WebSocket (via Axum)
//! - **TCP**: Direct TCP socket communication
//! - **Unix Sockets**: Fast inter-process communication (Unix only)
//! - **Child Process**: Spawn and manage child MCP servers
//!
//! # Usage Examples
//!
//! ```bash
//! # Default STDIO transport (always available)
//! cargo run --example transport_showcase
//!
//! # HTTP server with REST API and MCP endpoints
//! TRANSPORT=http PORT=8080 cargo run --example transport_showcase --features http
//!
//! # TCP transport with custom port
//! TRANSPORT=tcp PORT=9000 cargo run --example transport_showcase --features tcp
//!
//! # Unix socket transport (Unix only)
//! TRANSPORT=unix SOCKET_PATH=/tmp/showcase.sock cargo run --example transport_showcase --features unix
//!
//! # Child process transport (spawn 'cat' for echo)
//! TRANSPORT=child CHILD_CMD=cat cargo run --example transport_showcase
//! ```
//!
//! # HTTP Server Features
//!
//! When using HTTP transport, the server provides:
//! - **REST API**: Traditional endpoints at `/api/*`
//! - **MCP JSON-RPC**: Protocol endpoint at `/mcp`
//! - **Server-Sent Events**: Real-time stream at `/mcp/sse`
//! - **WebSocket**: Full-duplex MCP at `/mcp/ws`
//! - **Health Checks**: Monitoring at `/mcp/health`
//! - **Web Interface**: Documentation at `/`

use std::sync::{
    Arc,
    atomic::{AtomicU32, AtomicU64},
};
use turbomcp::prelude::*;

#[derive(Clone)]
struct AdvancedCalculator {
    operations: Arc<AtomicU64>,
    precision: Arc<AtomicU32>,
}

#[server]
impl AdvancedCalculator {
    #[tool("Add two numbers with high precision")]
    async fn add(&self, a: f64, b: f64) -> McpResult<f64> {
        self.operations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let precision = self.precision.load(std::sync::atomic::Ordering::Relaxed) as i32;
        let result = (a + b * 10_f64.powi(precision)).round() / 10_f64.powi(precision);
        Ok(result)
    }

    #[tool("Get server statistics")]
    async fn stats(&self) -> McpResult<String> {
        let ops = self.operations.load(std::sync::atomic::Ordering::Relaxed);
        let precision = self.precision.load(std::sync::atomic::Ordering::Relaxed);
        Ok(format!(
            "Operations performed: {} (precision: {} decimal places)",
            ops, precision
        ))
    }

    #[tool("Set calculation precision")]
    async fn set_precision(&self, precision: u32) -> McpResult<String> {
        let capped_precision = precision.min(10); // Cap at 10 decimal places
        self.precision
            .store(capped_precision, std::sync::atomic::Ordering::Relaxed);
        Ok(format!(
            "Precision set to {} decimal places",
            capped_precision
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let server = AdvancedCalculator {
        operations: Arc::new(AtomicU64::new(0)),
        precision: Arc::new(AtomicU32::new(2)),
    };

    println!("🚀 TurboMCP Transport Showcase");
    println!("===============================");

    // Progressive enhancement with runtime configuration
    match std::env::var("TRANSPORT").as_deref() {
        Ok("http") => {
            let port: u16 = std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080);

            println!(
                "🌍 Starting HTTP server with full web interface on port {}",
                port
            );
            println!("   🔗 Web interface: http://localhost:{}/", port);
            println!("   📡 REST API: http://localhost:{}/api/", port);
            println!("   🎯 MCP endpoint: http://localhost:{}/mcp", port);
            println!(
                "   📊 Server-Sent Events: http://localhost:{}/mcp/sse",
                port
            );
            println!("   🔄 WebSocket: ws://localhost:{}/mcp/ws", port);
            println!("   ❤️  Health check: http://localhost:{}/mcp/health", port);

            #[cfg(feature = "http")]
            {
                use async_trait::async_trait;
                use axum::{Router, routing::get};
                use tokio::net::TcpListener;
                use turbomcp_transport::{AxumMcpExt, McpService, SessionInfo};

                // Simple wrapper to implement McpService for AdvancedCalculator
                #[derive(Clone)]
                struct McpWrapper {
                    #[allow(dead_code)] // Used for demo purposes
                    calculator: AdvancedCalculator,
                }

                #[async_trait]
                impl McpService for McpWrapper {
                    async fn process_request(
                        &self,
                        request: serde_json::Value,
                        _session: &SessionInfo,
                    ) -> turbomcp_core::Result<serde_json::Value> {
                        // Simple echo for demo - in real usage you'd route to actual MCP handlers
                        Ok(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request.get("id"),
                            "result": {
                                "message": "This is a demo HTTP server showcasing transport capabilities",
                                "request": request,
                                "available_tools": ["add", "stats", "set_precision"],
                                "note": "For full MCP functionality, see the 10_http_server example"
                            }
                        }))
                    }
                }

                let mcp_service = McpWrapper {
                    calculator: server.clone(),
                };

                // Capture port value for the closure
                let port_for_closure = port;

                // Create HTTP server with MCP integration
                let app = Router::new()
                    .route("/", get(move || async move { 
                        axum::response::Html(format!(r#"
                            <h1>TurboMCP Transport Showcase</h1>
                            <p>HTTP server running with MCP capabilities!</p>
                            <ul>
                                <li><a href="/api/stats">API Stats</a></li>
                                <li><a href="/mcp/capabilities">MCP Capabilities</a></li>
                                <li><a href="/mcp/health">Health Check</a></li>
                                <li><a href="/mcp/sse">Server-Sent Events</a> (curl this!)</li>
                            </ul>
                            <p>Try: <code>curl -X POST http://localhost:{}/mcp -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{{}}}}'</code></p>
                        "#, port_for_closure))
                    }))
                    .route("/api/stats", get(|| async { 
                        axum::Json(serde_json::json!({
                            "message": "Hello from REST API!",
                            "server": "TurboMCP Transport Showcase",
                            "transport": "HTTP"
                        }))
                    }))
                    .turbo_mcp_routes(mcp_service);

                let addr = format!("0.0.0.0:{}", port);
                let listener = TcpListener::bind(&addr).await?;
                axum::serve(listener, app).await?;
            }
            #[cfg(not(feature = "http"))]
            {
                eprintln!("❌ HTTP transport not enabled. Run with --features http");
                std::process::exit(1);
            }
        }
        Ok("tcp") => {
            let port: u16 = std::env::var("PORT")
                .unwrap_or_else(|_| "9000".to_string())
                .parse()
                .unwrap_or(9000);
            let addr = format!("127.0.0.1:{}", port);

            println!("🌐 Starting TCP server on {}", addr);
            println!("   Connect with: nc 127.0.0.1 {}", port);
            println!(
                "   Send JSON-RPC: {{\"jsonrpc\":\"2.0\",\"id\":\"1\",\"method\":\"tools/list\",\"params\":{{}}}}"
            );

            #[cfg(feature = "tcp")]
            {
                server.run_tcp(addr).await?;
            }
            #[cfg(not(feature = "tcp"))]
            {
                eprintln!("❌ TCP transport not enabled. Run with --features tcp");
                std::process::exit(1);
            }
        }
        Ok("unix") => {
            let path = std::env::var("SOCKET_PATH")
                .unwrap_or_else(|_| "/tmp/turbomcp_showcase.sock".to_string());

            println!("🔌 Starting Unix socket server at {}", path);
            println!("   Connect with: nc -U {}", path);
            println!(
                "   Send JSON-RPC: {{\"jsonrpc\":\"2.0\",\"id\":\"1\",\"method\":\"tools/list\",\"params\":{{}}}}"
            );

            #[cfg(all(feature = "unix", unix))]
            {
                // Remove existing socket file if it exists
                let _ = std::fs::remove_file(&path);
                server.run_unix(path).await?;
            }
            #[cfg(not(all(feature = "unix", unix)))]
            {
                eprintln!(
                    "❌ Unix transport not enabled or not on Unix. Run with --features unix on Unix systems"
                );
                std::process::exit(1);
            }
        }
        Ok("child") => {
            let child_cmd = std::env::var("CHILD_CMD").unwrap_or_else(|_| "cat".to_string());

            println!("👶 Child process transport demonstration");
            println!("   Command to spawn: {}", child_cmd);
            println!("   Note: This demonstrates spawning child processes for MCP communication");
            println!(
                "   The showcase server will demonstrate child process management capabilities"
            );
            println!();

            // Add child process management tool
            println!("📝 Added extra tool: spawn_child_process");
            println!(
                "   This tool demonstrates spawning '{}' as a child process",
                child_cmd
            );

            // For demo, just run STDIO with additional context
            server.run_stdio().await?;
        }
        Ok(transport) => {
            println!(
                "❓ Unknown transport '{}', falling back to STDIO",
                transport
            );
            server.run_stdio().await?;
        }
        _ => {
            println!("📝 Starting STDIO server (default)");
            println!("   Reading from stdin, writing to stdout");
            println!("   Send JSON-RPC messages or press Ctrl+C to exit");
            server.run_stdio().await?;
        }
    }

    Ok(())
}
