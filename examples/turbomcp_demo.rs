//! TurboMCP Demo - Ergonomic MCP server example

use turbomcp::prelude::*;

/// Demo server showcasing TurboMCP capabilities
#[turbomcp::server]
struct DemoServer {
    counter: std::sync::atomic::AtomicI32,
}

impl DemoServer {
    /// Create a new demo server
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicI32::new(0),
        }
    }

    /// Add two numbers together
    #[tool("Add two numbers and return the result")]
    async fn add(&self, a: i32, b: i32) -> McpResult<i32> {
        // Increment counter to track usage
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(a + b)
    }

    /// Get server statistics  
    #[tool("Get server statistics")]
    async fn stats(&self) -> McpResult<String> {
        let count = self.counter.load(std::sync::atomic::Ordering::SeqCst);
        Ok(format!("Operations performed: {}", count))
    }

    /// Echo a message with context
    #[tool("Echo a message with context information")]
    async fn echo(&self, ctx: Context, message: String) -> McpResult<String> {
        ctx.info(&format!("Processing echo request: {}", message))
            .await?;
        Ok(format!("Echo: {}", message))
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create and run the server
    let server = DemoServer::new();
    server.run_stdio().await
}
