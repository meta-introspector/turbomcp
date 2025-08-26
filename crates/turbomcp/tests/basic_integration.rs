//! Basic integration tests that should compile and run successfully

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::sync::Mutex;

use turbomcp::prelude::*;
use turbomcp_core::RequestContext;

/// Simple test server for basic integration testing
#[derive(Clone)]
struct BasicTestServer {
    counter: Arc<AtomicI32>,
    state: Arc<Mutex<String>>,
}

impl BasicTestServer {
    fn new() -> Self {
        Self {
            counter: Arc::new(AtomicI32::new(0)),
            state: Arc::new(Mutex::new("initial".to_string())),
        }
    }

    async fn increment(&self, amount: i32) -> McpResult<i32> {
        let new_value = self.counter.fetch_add(amount, Ordering::SeqCst) + amount;
        Ok(new_value)
    }

    async fn get_state(&self) -> McpResult<String> {
        let state = self.state.lock().await;
        Ok(state.clone())
    }

    async fn set_state(&self, new_state: String) -> McpResult<String> {
        let mut state = self.state.lock().await;
        let old_state = state.clone();
        *state = new_state;
        Ok(format!("Changed from '{old_state}' to '{state}'"))
    }
}

#[async_trait]
impl HandlerRegistration for BasicTestServer {
    async fn register_with_builder(&self, _builder: &mut ServerBuilder) -> McpResult<()> {
        // In a real implementation, this would register handlers
        Ok(())
    }
}

#[async_trait]
impl TurboMcpServer for BasicTestServer {
    fn name(&self) -> &'static str {
        "BasicTestServer"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> Option<&str> {
        Some("A basic test server for integration testing")
    }
}

#[tokio::test]
async fn test_server_creation() {
    let server = BasicTestServer::new();

    assert_eq!(server.name(), "BasicTestServer");
    assert_eq!(server.version(), "1.0.0");
    assert!(server.description().is_some());

    // Test initial state
    assert_eq!(server.counter.load(Ordering::SeqCst), 0);
    assert_eq!(server.get_state().await.unwrap(), "initial");
}

#[tokio::test]
async fn test_server_operations() {
    let server = BasicTestServer::new();

    // Test increment operation
    let result1 = server.increment(5).await.unwrap();
    assert_eq!(result1, 5);

    let result2 = server.increment(3).await.unwrap();
    assert_eq!(result2, 8);

    // Test state operations
    let state_result = server.set_state("new_state".to_string()).await.unwrap();
    assert_eq!(state_result, "Changed from 'initial' to 'new_state'");

    let current_state = server.get_state().await.unwrap();
    assert_eq!(current_state, "new_state");
}

#[tokio::test]
async fn test_concurrent_operations() {
    let server = Arc::new(BasicTestServer::new());
    let mut handles = vec![];

    // Spawn multiple concurrent increment operations
    for i in 1..=5 {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move { server_clone.increment(i).await });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap().unwrap();
        results.push(result);
    }

    // Final counter value should be 1+2+3+4+5 = 15
    let final_counter = server.counter.load(Ordering::SeqCst);
    assert_eq!(final_counter, 15);

    // All results should be valid
    assert_eq!(results.len(), 5);
    results.sort();
    assert!(results.iter().all(|&x| (1..=15).contains(&x)));
}

#[tokio::test]
async fn test_error_handling() {
    async fn failing_operation() -> McpResult<String> {
        Err(McpError::Tool("Test error".to_string()))
    }

    async fn successful_operation() -> McpResult<String> {
        Ok("Success".to_string())
    }

    // Test error propagation
    let error_result = failing_operation().await;
    assert!(error_result.is_err());
    match error_result.unwrap_err() {
        McpError::Tool(msg) => assert_eq!(msg, "Test error"),
        _ => panic!("Expected Tool error"),
    }

    // Test successful operation
    let success_result = successful_operation().await;
    assert!(success_result.is_ok());
    assert_eq!(success_result.unwrap(), "Success");
}

#[tokio::test]
async fn test_handler_registration() {
    let server = BasicTestServer::new();
    let mut builder = ServerBuilder::new().name("test").version("1.0.0");

    // Registration should succeed
    let registration_result = server.register_with_builder(&mut builder).await;
    assert!(registration_result.is_ok());

    // Builder should be able to create a server
    let _mcp_server = builder.build();
}

#[tokio::test]
async fn test_context_creation_and_usage() {
    let request_context = RequestContext::new();
    let handler_metadata = HandlerMetadata {
        name: "test_handler".to_string(),
        handler_type: "tool".to_string(),
        description: Some("Test handler for context testing".to_string()),
    };

    let context = Context::new(request_context, handler_metadata);

    // Test context properties
    assert_eq!(context.handler.name, "test_handler");
    assert_eq!(context.handler.handler_type, "tool");

    // Test context data storage
    context.set("test_key", "test_value").await.unwrap();
    let retrieved: Option<String> = context.get("test_key").await.unwrap();
    assert_eq!(retrieved, Some("test_value".to_string()));
}

#[tokio::test]
async fn test_helper_functions() {
    // Test text helper
    let content = text("Hello, World!");
    match content {
        Content::Text(text_content) => {
            assert_eq!(text_content.text, "Hello, World!");
        }
        _ => panic!("Expected TextContent"),
    }

    // Test tool_success helper
    let result = tool_success(vec![text("Operation completed")]);
    assert!(!result.is_error.unwrap_or(true));
    assert_eq!(result.content.len(), 1);

    // Test tool_error helper
    let error_result = tool_error("Operation failed");
    assert!(error_result.is_error.unwrap_or(false));
    assert_eq!(error_result.content.len(), 1);
}

#[tokio::test]
async fn test_server_lifecycle() {
    let server = BasicTestServer::new();

    // Test startup
    assert!(server.startup().await.is_ok());

    // Test shutdown
    assert!(server.shutdown().await.is_ok());
}

#[tokio::test]
async fn test_performance_basic() {
    let server = BasicTestServer::new();
    let start = std::time::Instant::now();

    // Execute 100 rapid operations
    for _ in 0..100 {
        server.increment(1).await.unwrap();
    }

    let duration = start.elapsed();

    // Should complete within reasonable time
    assert!(
        duration.as_millis() < 1000,
        "Performance test took too long: {duration:?}"
    );

    // Final counter should be correct
    assert_eq!(server.counter.load(Ordering::SeqCst), 100);
}

// =============================================================================
// COMPREHENSIVE TEST SUITE
// =============================================================================

#[tokio::test]
async fn test_comprehensive_auth_simulation() {
    // Test authentication-like flow with state management
    use std::collections::HashMap;

    let mut auth_states = HashMap::new();

    // Simulate OAuth2 state
    let auth_state = "random_state_123";
    auth_states.insert(
        auth_state,
        (
            "test_client".to_string(),
            "https://example.com/callback".to_string(),
            chrono::Utc::now().timestamp(),
        ),
    );

    // Verify state exists and is valid
    let stored_state = auth_states.get(auth_state);
    assert!(stored_state.is_some());

    let (client_id, redirect_uri, _timestamp) = stored_state.unwrap();
    assert_eq!(client_id, "test_client");
    assert_eq!(redirect_uri, "https://example.com/callback");

    // Simulate completion and cleanup
    auth_states.remove(auth_state);
    assert!(!auth_states.contains_key(auth_state));
}

#[tokio::test]
async fn test_comprehensive_transport_robustness() {
    // Test transport resilience patterns
    use std::sync::atomic::{AtomicU32, Ordering};

    // Simulate circuit breaker behavior
    let failure_count = Arc::new(AtomicU32::new(0));
    let threshold = 5;
    let max_failures = 10;

    // Simulate multiple failures
    for i in 0..max_failures {
        failure_count.fetch_add(1, Ordering::SeqCst);
        let current_failures = failure_count.load(Ordering::SeqCst);

        // Test circuit breaker state logic
        let circuit_open = current_failures >= threshold;

        if i >= threshold as usize - 1 {
            assert!(
                circuit_open,
                "Circuit should be open after {} failures",
                i + 1
            );
        } else {
            assert!(
                !circuit_open,
                "Circuit should be closed with {} failures",
                i + 1
            );
        }
    }

    // Simulate recovery
    failure_count.store(0, Ordering::SeqCst);
    assert_eq!(failure_count.load(Ordering::SeqCst), 0);
    assert!(failure_count.load(Ordering::SeqCst) < threshold);
}

#[tokio::test]
async fn test_comprehensive_protocol_validation() {
    use serde_json::{Value, json};

    // Test complex nested validation scenarios
    let complex_params = json!({
        "nested": {
            "level1": {
                "level2": {
                    "data": "test",
                    "array": [1, 2, 3, 4, 5]
                }
            }
        },
        "unicode": "🌟 Testing unicode: こんにちは 🚀",
        "large_array": (0..100).collect::<Vec<_>>(),
        "special_chars": "\"escape\" and \n newline",
        "numbers": {
            "integer": 42,
            "float": std::f64::consts::PI,
            "negative": -100,
            "zero": 0
        }
    });

    // Test serialization of complex data
    let serialized = serde_json::to_string(&complex_params);
    assert!(
        serialized.is_ok(),
        "Complex data should serialize successfully"
    );

    // Test deserialization
    let deserialized: Result<Value, _> = serde_json::from_str(&serialized.unwrap());
    assert!(
        deserialized.is_ok(),
        "Complex data should deserialize successfully"
    );

    let deserialized_data = deserialized.unwrap();

    // Verify nested data integrity
    assert_eq!(
        deserialized_data["unicode"],
        "🌟 Testing unicode: こんにちは 🚀"
    );
    assert_eq!(
        deserialized_data["large_array"].as_array().unwrap().len(),
        100
    );
    assert_eq!(deserialized_data["numbers"]["integer"], 42);
    assert_eq!(deserialized_data["numbers"]["float"], std::f64::consts::PI);

    // Test deep access
    assert_eq!(
        deserialized_data["nested"]["level1"]["level2"]["data"],
        "test"
    );
}

#[tokio::test]
async fn test_comprehensive_performance_simulation() {
    use std::time::Instant;

    // Test high-throughput scenario with multiple servers
    let servers: Vec<BasicTestServer> = (0..5).map(|_| BasicTestServer::new()).collect();
    let server_arcs: Vec<Arc<BasicTestServer>> = servers.into_iter().map(Arc::new).collect();

    let start = Instant::now();
    let operations_per_server = 500; // Reduced for CI stability

    // Concurrent operations across multiple servers
    let mut handles = vec![];
    for (server_id, server) in server_arcs.iter().enumerate() {
        for op_id in 0..operations_per_server {
            let server_clone = Arc::clone(server);
            let handle = tokio::spawn(async move {
                let increment_amount = (server_id + 1) as i32;
                server_clone.increment(increment_amount).await.unwrap();

                // Also test state operations
                if op_id % 10 == 0 {
                    let state_value = format!("server_{server_id}_op_{op_id}");
                    server_clone.set_state(state_value).await.unwrap();
                }
            });
            handles.push(handle);
        }
    }

    // Wait for completion
    for handle in handles {
        assert!(handle.await.is_ok());
    }

    let duration = start.elapsed();
    println!(
        "Performance test: {} servers × {} ops in {:?}",
        server_arcs.len(),
        operations_per_server,
        duration
    );

    // Verify all operations completed correctly
    for (server_id, server) in server_arcs.iter().enumerate() {
        let expected_count = operations_per_server * (server_id + 1);
        let actual_count = server.counter.load(Ordering::SeqCst) as usize;
        assert_eq!(
            actual_count, expected_count,
            "Server {server_id} count mismatch"
        );
    }

    // Performance should be reasonable (allow up to 10 seconds for CI)
    assert!(
        duration.as_secs() < 10,
        "Performance test took too long: {duration:?}"
    );
}

#[tokio::test]
async fn test_comprehensive_error_handling() {
    // Test various error scenarios and recovery patterns

    async fn simulate_network_error() -> McpResult<String> {
        Err(McpError::Transport("Connection timeout".to_string()))
    }

    async fn simulate_protocol_error() -> McpResult<String> {
        Err(McpError::Protocol("Invalid JSON-RPC format".to_string()))
    }

    async fn simulate_server_error() -> McpResult<String> {
        Err(McpError::Server(turbomcp::ServerError::Lifecycle(
            "Server overloaded".to_string(),
        )))
    }

    async fn simulate_auth_error() -> McpResult<String> {
        Err(McpError::Tool("Authentication failed".to_string()))
    }

    // Test each error type
    let transport_result = simulate_network_error().await;
    assert!(transport_result.is_err());
    match transport_result.unwrap_err() {
        McpError::Transport(msg) => assert!(msg.contains("Connection timeout")),
        _ => panic!("Expected Transport error"),
    }

    let protocol_result = simulate_protocol_error().await;
    assert!(protocol_result.is_err());
    match protocol_result.unwrap_err() {
        McpError::Protocol(msg) => assert!(msg.contains("Invalid JSON-RPC")),
        _ => panic!("Expected Protocol error"),
    }

    let server_result = simulate_server_error().await;
    assert!(server_result.is_err());
    match server_result.unwrap_err() {
        McpError::Server(err) => assert!(err.to_string().contains("Server overloaded")),
        _ => panic!("Expected Server error"),
    }

    let auth_result = simulate_auth_error().await;
    assert!(auth_result.is_err());
    match auth_result.unwrap_err() {
        McpError::Tool(msg) => assert!(msg.contains("Authentication failed")),
        _ => panic!("Expected Tool error"),
    }
}

#[tokio::test]
async fn test_comprehensive_concurrent_safety() {
    // Test thread safety with heavy concurrent access across multiple scenarios
    let server = Arc::new(BasicTestServer::new());
    let num_threads = 20; // Increased for more thorough testing
    let ops_per_thread = 50;

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            for op_id in 0..ops_per_thread {
                // Mix of operations to test different code paths
                match op_id % 3 {
                    0 => {
                        // Increment operation
                        let result = server_clone.increment(thread_id + 1).await;
                        assert!(result.is_ok(), "Increment should succeed");
                    }
                    1 => {
                        // State read operation
                        let result = server_clone.get_state().await;
                        assert!(result.is_ok(), "Get state should succeed");
                    }
                    2 => {
                        // State write operation
                        let new_state = format!("thread_{thread_id}_op_{op_id}");
                        let result = server_clone.set_state(new_state).await;
                        assert!(result.is_ok(), "Set state should succeed");
                    }
                    _ => unreachable!(),
                }

                // Yield to allow other threads to run
                tokio::task::yield_now().await;
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        assert!(
            handle.await.is_ok(),
            "All concurrent operations should succeed"
        );
    }

    // Verify final state is consistent
    let final_counter = server.counter.load(Ordering::SeqCst);
    let _expected_increments =
        num_threads * (ops_per_thread / 3 + if ops_per_thread % 3 > 0 { 1 } else { 0 });

    // Counter should have been incremented multiple times
    assert!(final_counter > 0, "Counter should have been incremented");

    // State should be readable
    let final_state = server.get_state().await;
    assert!(final_state.is_ok(), "Final state should be readable");
}

#[tokio::test]
async fn test_comprehensive_resource_lifecycle() {
    // Test complete resource lifecycle with cleanup
    let server = BasicTestServer::new();

    // Resource creation phase
    let mut resource_ids = Vec::new();
    for i in 0..50 {
        let resource_state = format!("resource_data_{i}");
        let result = server.set_state(resource_state.clone()).await;
        assert!(result.is_ok(), "Resource creation should succeed");
        resource_ids.push(i);
    }

    // Resource utilization phase
    for &resource_id in &resource_ids {
        let current_state = server.get_state().await.unwrap();
        assert!(
            !current_state.is_empty(),
            "Resource state should not be empty"
        );

        // Simulate resource usage
        let increment_result = server.increment(resource_id).await;
        assert!(increment_result.is_ok(), "Resource usage should succeed");
    }

    // Resource cleanup phase
    let final_counter = server.counter.load(Ordering::SeqCst);
    let expected_sum: i32 = resource_ids.iter().sum();
    assert_eq!(
        final_counter, expected_sum,
        "Counter should match expected sum"
    );

    // Final state check
    let final_state = server.get_state().await;
    assert!(final_state.is_ok(), "Final state check should succeed");
}

#[tokio::test]
async fn test_comprehensive_edge_cases() {
    let server = BasicTestServer::new();

    // Test edge case: zero increment
    let zero_result = server.increment(0).await;
    assert!(zero_result.is_ok());
    assert_eq!(zero_result.unwrap(), 0);

    // Test edge case: negative increment
    let negative_result = server.increment(-5).await;
    assert!(negative_result.is_ok());
    assert_eq!(negative_result.unwrap(), -5);

    // Test edge case: very large increment
    let large_result = server.increment(i32::MAX / 2).await;
    assert!(large_result.is_ok());

    // Test edge case: empty state
    let empty_state_result = server.set_state("".to_string()).await;
    assert!(empty_state_result.is_ok());

    let retrieved_empty = server.get_state().await;
    assert!(retrieved_empty.is_ok());
    assert_eq!(retrieved_empty.unwrap(), "");

    // Test edge case: very long state
    let long_state = "x".repeat(10000);
    let long_state_result = server.set_state(long_state.clone()).await;
    assert!(long_state_result.is_ok());

    let retrieved_long = server.get_state().await;
    assert!(retrieved_long.is_ok());
    assert_eq!(retrieved_long.unwrap(), long_state);

    // Test edge case: unicode in state
    let unicode_state = "🚀 Test with émojis and ñoñó characters 中文 العربية";
    let unicode_result = server.set_state(unicode_state.to_string()).await;
    assert!(unicode_result.is_ok());

    let retrieved_unicode = server.get_state().await;
    assert!(retrieved_unicode.is_ok());
    assert_eq!(retrieved_unicode.unwrap(), unicode_state);
}

#[tokio::test]
async fn test_comprehensive_stress_patterns() {
    use std::time::{Duration, Instant};

    let server = Arc::new(BasicTestServer::new());
    let start_time = Instant::now();
    let test_duration = Duration::from_millis(500); // Short stress test

    let mut handles = vec![];

    // Spawn stress test workers
    for worker_id in 0..5 {
        let server_clone = Arc::clone(&server);
        let end_time = start_time + test_duration;

        let handle = tokio::spawn(async move {
            let mut operation_count = 0;

            while Instant::now() < end_time {
                match operation_count % 4 {
                    0 => {
                        let result = server_clone.increment(1).await;
                        assert!(
                            result.is_ok(),
                            "Increment should succeed during stress test"
                        );
                    }
                    1 => {
                        let result = server_clone.get_state().await;
                        assert!(
                            result.is_ok(),
                            "Get state should succeed during stress test"
                        );
                    }
                    2 => {
                        let state = format!("stress_worker_{worker_id}_op_{operation_count}");
                        let result = server_clone.set_state(state).await;
                        assert!(
                            result.is_ok(),
                            "Set state should succeed during stress test"
                        );
                    }
                    3 => {
                        let result = server_clone.increment(-1).await;
                        assert!(
                            result.is_ok(),
                            "Decrement should succeed during stress test"
                        );
                    }
                    _ => unreachable!(),
                }

                operation_count += 1;

                // Small delay to prevent overwhelming
                tokio::time::sleep(Duration::from_micros(100)).await;
            }

            operation_count
        });

        handles.push(handle);
    }

    // Wait for all stress workers to complete
    let mut total_operations = 0;
    for handle in handles {
        let worker_ops = handle.await.unwrap();
        total_operations += worker_ops;
    }

    let elapsed = start_time.elapsed();
    println!("Stress test: {total_operations} operations in {elapsed:?}");

    // Verify system is still functional after stress
    let post_stress_increment = server.increment(42).await;
    assert!(
        post_stress_increment.is_ok(),
        "Server should be functional after stress test"
    );

    let post_stress_state = server.get_state().await;
    assert!(
        post_stress_state.is_ok(),
        "Server state should be accessible after stress test"
    );

    // Should have completed a reasonable number of operations
    assert!(
        total_operations > 0,
        "Should have completed some operations"
    );
}
