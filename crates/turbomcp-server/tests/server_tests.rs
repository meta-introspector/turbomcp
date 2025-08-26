//! Tests for the MCP server implementation

use turbomcp_server::prelude::*;

#[tokio::test]
async fn test_server_creation() {
    let config = ServerConfig::default();
    let server = McpServer::new(config);

    // Test that server can be created
    assert!(format!("{server:?}").contains("McpServer"));
}

#[tokio::test]
async fn test_server_builder() {
    let builder = ServerBuilder::new();
    let server = builder.build();

    // Test builder pattern works
    assert!(format!("{server:?}").contains("McpServer"));
}

#[tokio::test]
async fn test_server_with_config() {
    let config = ServerConfig {
        name: "test-server".to_string(),
        ..Default::default()
    };

    let server = McpServer::new(config);
    assert!(format!("{server:?}").contains("McpServer"));
}

#[test]
fn test_health_status_creation() {
    let health_status = HealthStatus::healthy();
    assert!(health_status.healthy);

    let unhealthy_status = HealthStatus::unhealthy();
    assert!(!unhealthy_status.healthy);
}
