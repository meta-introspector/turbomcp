//! Simple test to verify macro basics work

use turbomcp_macros::{server, tool};

#[derive(Clone)]
struct SimpleServer;

#[server(name = "Simple", version = "1.0.0")]
impl SimpleServer {
    #[tool("Test tool")]
    async fn test_tool(&self) -> turbomcp::McpResult<String> {
        Ok("test".to_string())
    }
}

#[test]
fn test_macro_compiles() {
    let _server = SimpleServer;
    // Macro compilation validated by successful instantiation
}

#[tokio::test]
async fn test_tool_method_exists() {
    let server = SimpleServer;
    // The original method should still exist
    let result = server.test_tool().await;
    assert_eq!(result.unwrap(), "test");
}
