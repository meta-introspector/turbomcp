//! README example validation - ensures the main README example actually compiles

use std::sync::{Arc, atomic::AtomicU64};
use turbomcp::prelude::*;

#[derive(Clone)]
struct Calculator {
    operations: Arc<AtomicU64>,
}

#[server]
impl Calculator {
    #[tool("Add two numbers")]
    async fn add(&self, ctx: Context, a: i32, b: i32) -> McpResult<i32> {
        // Context injection provides automatic:
        // - Request correlation and distributed tracing
        // - Structured logging with metadata
        // - Performance monitoring and metrics collection
        ctx.info(&format!("Adding {} + {}", a, b)).await?;

        self.operations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let result = a + b;
        ctx.info(&format!("Addition result: {}", result)).await?;
        Ok(result)
    }

    #[tool("Get server statistics")]
    async fn stats(&self, ctx: Context) -> McpResult<String> {
        ctx.info("Gathering server statistics with full observability")
            .await?;

        let ops = self.operations.load(std::sync::atomic::Ordering::Relaxed);
        let stats = format!("Operations performed: {}", ops);

        ctx.info(&format!("Statistics generated: {}", stats))
            .await?;
        Ok(stats)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Calculator {
        operations: Arc::new(AtomicU64::new(0)),
    };
    server.run_stdio().await?;
    Ok(())
}
