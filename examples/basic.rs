//! Basic MCP server example with configuration and setup
//!
//! This example shows:
//! - Basic server configuration
//! - Logging setup
//! - Server lifecycle management
//!
//! Run with: `cargo run --example basic`

use std::sync::Arc;
use turbomcp_core::Result;
use turbomcp_server::{McpServer, ServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging for better debugging
    // Set RUST_LOG=debug for more detailed output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .with_target(false)
        .with_thread_ids(true)
        .init();

    println!("🚀 Starting Basic TurboMCP Server Example");
    println!("   This example demonstrates basic server setup");
    println!("   For a working server with tools, try: cargo run --example turbomcp_demo");
    println!();

    tracing::info!("Initializing basic MCP server");

    // Create server configuration with detailed metadata
    let config = ServerConfig {
        name: "basic-example".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Basic MCP server example demonstrating configuration".to_string()),
        
        // Optional: Configure server capabilities
        // max_request_size: Some(1024 * 1024), // 1MB
        // timeout_ms: Some(30000), // 30 seconds
        
        ..Default::default()
    };

    // Create the server instance
    let server = Arc::new(McpServer::new(config));
    
    tracing::info!("Server configured successfully");
    tracing::info!("Server name: {}", server.config().name);
    tracing::info!("Server version: {}", server.config().version);
    
    // In a real application, you would run the server:
    // server.run_stdio().await?;
    
    // For this example, we'll just demonstrate the server is ready
    println!("✅ Basic server created and configured successfully!");
    println!("   Server name: {}", server.config().name);
    println!("   Server version: {}", server.config().version);
    println!();
    println!("💡 Next steps:");
    println!("   • Add tools with the #[tool] macro");
    println!("   • Add resources with the #[resource] macro"); 
    println!("   • Add prompts with the #[prompt] macro");
    println!("   • See comprehensive_macros.rs for all features");

    tracing::info!("Basic server example completed successfully");

    Ok(())
}
