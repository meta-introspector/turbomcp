//! Async server tests focusing on lifecycle and health functionality

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use turbomcp_server::{config::ServerConfig, server::McpServer};

// ========== Server Health and Lifecycle Tests ==========

#[tokio::test]
async fn test_server_health_initial_state() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = false; // Disable to avoid complexity
    let server = McpServer::new(config);

    let health = server.health().await;
    // Initial state should have healthy=true from default lifecycle
    assert!(health.healthy);
}

#[tokio::test]
async fn test_server_health_after_start() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = false;
    let server = McpServer::new(config);

    // Start the lifecycle
    server.lifecycle().start().await;

    let health = server.health().await;
    assert!(health.healthy);
    assert!(health.details.is_empty()); // Should start with no health checks
}

#[tokio::test]
async fn test_server_health_after_shutdown() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = false;
    let server = McpServer::new(config);

    server.lifecycle().start().await;
    server.lifecycle().shutdown().await;

    let health = server.health().await;
    // Health is maintained separately from state, so check timestamp freshness
    assert!(health.timestamp.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn test_server_lifecycle_state_transitions() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = false;
    let server = McpServer::new(config);

    // Test state transitions through lifecycle
    let lifecycle = server.lifecycle();

    // Initial state should be Starting
    use turbomcp_server::lifecycle::ServerState;
    let initial_state = lifecycle.state().await;
    assert_eq!(initial_state, ServerState::Starting);

    lifecycle.start().await;
    let running_state = lifecycle.state().await;
    assert_eq!(running_state, ServerState::Running);

    lifecycle.shutdown().await;
    let shutdown_state = lifecycle.state().await;
    assert_eq!(shutdown_state, ServerState::ShuttingDown);
}

// ========== Integration Tests ==========

#[tokio::test]
async fn test_server_lifecycle_integration() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = false;
    let server = McpServer::new(config);

    // Test complete lifecycle
    let initial_health = server.health().await;
    assert!(initial_health.healthy);

    server.lifecycle().start().await;
    let running_health = server.health().await;
    assert!(running_health.healthy);

    server.lifecycle().shutdown().await;
    let shutdown_health = server.health().await;
    // Health status is maintained independently, check timestamp freshness
    assert!(shutdown_health.timestamp.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn test_server_with_configured_rate_limiting_integration() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = true;
    config.rate_limiting.requests_per_second = 5;
    config.rate_limiting.burst_capacity = 10;

    let server = McpServer::new(config);

    // Test that rate limiting configuration is preserved
    assert!(server.config().rate_limiting.enabled);
    assert_eq!(server.config().rate_limiting.requests_per_second, 5);
    assert_eq!(server.config().rate_limiting.burst_capacity, 10);

    // Start the server and verify health
    server.lifecycle().start().await;
    let health = server.health().await;
    assert!(health.healthy);
}

#[tokio::test]
async fn test_multiple_servers_independence() {
    let mut config1 = ServerConfig {
        name: "server1".to_string(),
        ..Default::default()
    };
    config1.rate_limiting.enabled = false;

    let mut config2 = ServerConfig {
        name: "server2".to_string(),
        ..Default::default()
    };
    config2.rate_limiting.enabled = false;

    let server1 = McpServer::new(config1);
    let server2 = McpServer::new(config2);

    // Verify they are independent
    assert_eq!(server1.config().name, "server1");
    assert_eq!(server2.config().name, "server2");

    // Start them independently
    server1.lifecycle().start().await;
    assert!(server1.health().await.healthy);
    assert!(server2.health().await.healthy); // Both start healthy

    server2.lifecycle().start().await;
    assert!(server1.health().await.healthy);
    assert!(server2.health().await.healthy);
}

// ========== Performance and Stress Tests ==========

#[tokio::test]
async fn test_server_operations_with_timeout() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = false;
    let server = McpServer::new(config);

    // Test that basic operations complete within reasonable time
    let health_result = timeout(Duration::from_millis(100), server.health()).await;
    assert!(health_result.is_ok());

    let lifecycle_start = timeout(Duration::from_millis(100), server.lifecycle().start()).await;
    assert!(lifecycle_start.is_ok());

    let lifecycle_shutdown =
        timeout(Duration::from_millis(100), server.lifecycle().shutdown()).await;
    assert!(lifecycle_shutdown.is_ok());
}

#[tokio::test]
async fn test_concurrent_server_operations() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = false;
    let server = Arc::new(McpServer::new(config));

    let mut handles = Vec::new();

    // Spawn multiple concurrent health checks
    for _ in 0..10 {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move { server_clone.health().await });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let health = handle.await.unwrap();
        // All should return valid health status
        assert!(health.timestamp.elapsed() < Duration::from_secs(1));
    }
}

// ========== Rate Limiting Tests ==========

#[tokio::test]
async fn test_server_creation_with_rate_limiting_enabled() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = true;
    config.rate_limiting.requests_per_second = 10;
    config.rate_limiting.burst_capacity = 20;

    let server = McpServer::new(config);
    assert!(server.config().rate_limiting.enabled);
    assert_eq!(server.config().rate_limiting.requests_per_second, 10);
    assert_eq!(server.config().rate_limiting.burst_capacity, 20);
}

#[tokio::test]
async fn test_server_with_full_config() {
    let mut config = ServerConfig {
        name: "full-featured".to_string(),
        version: "2.0.0".to_string(),
        description: Some("A fully configured server".to_string()),
        ..Default::default()
    };
    config.rate_limiting.enabled = true;
    config.rate_limiting.requests_per_second = 100;
    config.rate_limiting.burst_capacity = 200;

    let server = McpServer::new(config);
    assert_eq!(server.config().name, "full-featured");
    assert_eq!(server.config().version, "2.0.0");
    assert_eq!(
        server.config().description,
        Some("A fully configured server".to_string())
    );
    assert!(server.config().rate_limiting.enabled);
    assert_eq!(server.config().rate_limiting.requests_per_second, 100);
    assert_eq!(server.config().rate_limiting.burst_capacity, 200);
}

#[tokio::test]
async fn test_server_with_extreme_rate_limiting_values() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = true;
    config.rate_limiting.requests_per_second = 1; // Very low
    config.rate_limiting.burst_capacity = 1; // Very low

    let server = McpServer::new(config);
    assert!(server.config().rate_limiting.enabled);
    assert_eq!(server.config().rate_limiting.requests_per_second, 1);
    assert_eq!(server.config().rate_limiting.burst_capacity, 1);
}

#[tokio::test]
async fn test_server_with_high_rate_limiting_values() {
    let mut config = ServerConfig::default();
    config.rate_limiting.enabled = true;
    config.rate_limiting.requests_per_second = 10000; // Very high
    config.rate_limiting.burst_capacity = 50000; // Very high

    let server = McpServer::new(config);
    assert!(server.config().rate_limiting.enabled);
    assert_eq!(server.config().rate_limiting.requests_per_second, 10000);
    assert_eq!(server.config().rate_limiting.burst_capacity, 50000);
}
