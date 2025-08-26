//! Tests for the mcp-server library functions

use turbomcp_server::prelude::*;

#[test]
fn test_default_config() {
    let config = default_config();
    assert!(!config.name.is_empty()); // Basic existence check
}

#[tokio::test]
async fn test_server_builder_creation() {
    let builder = server();
    // Test that we can create a builder instance
    let _server = builder.build();
}

#[test]
fn test_constants() {
    assert_eq!(turbomcp_server::SERVER_NAME, "turbomcp-server");
    assert!(!turbomcp_server::SERVER_VERSION.is_empty());
}

#[test]
fn test_prelude_imports() {
    // Test that prelude items are accessible
    let _config = default_config();
    let _builder = server();

    // Test type availability - just test compilation
    let _health_status: Option<HealthStatus> = None;
    let _server_error: Option<ServerError> = None;
}
