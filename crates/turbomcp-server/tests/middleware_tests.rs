//! Core middleware functionality tests - consolidated from middleware_tests.rs
//!
//! This refactored test suite eliminates duplication while maintaining comprehensive coverage
//! of the middleware system's core functionality.

mod common;

use async_trait::async_trait;
use common::*;
use turbomcp_core::RequestContext;
use turbomcp_protocol::jsonrpc::*;
use turbomcp_server::ServerResult;
use turbomcp_server::middleware::*;

// ============================================================================
// Configuration Tests - Using DRY Macros
// ============================================================================

test_default_config!(StackConfig, enable_tracing, true);
test_debug_config!(StackConfig);
test_clone_config!(StackConfig);

#[test]
fn test_stack_config_custom() {
    let config = StackConfig {
        enable_tracing: false,
        timeout_ms: 10000,
        enable_metrics: false,
        enable_recovery: true,
    };

    assert!(!config.enable_tracing);
    assert_eq!(config.timeout_ms, 10000);
    assert!(!config.enable_metrics);
    assert!(config.enable_recovery);
}

// ============================================================================
// Core Middleware Stack Tests
// ============================================================================

#[test]
fn test_middleware_stack_creation() {
    // Test various ways to create middleware stack
    let stack1 = MiddlewareStack::new();
    let stack2 = MiddlewareStack::default();
    let stack3 = MiddlewareStack::with_config(StackConfig::default());

    // All should be functionally equivalent - just verify they can be created
    let _ = (stack1, stack2, stack3);
}

#[tokio::test]
async fn test_middleware_stack_basic_flow() {
    let mut stack = MiddlewareStack::new();

    // Test empty stack
    let request = create_test_request();
    let ctx = create_test_context();
    let result = stack.process_request(request, ctx).await;
    assert!(result.is_ok());

    // Add middleware and test
    stack.add(TestMiddleware::new("test1"));
    let request = create_test_request();
    let ctx = create_test_context();
    let (_, processed_ctx) = stack.process_request(request, ctx).await.unwrap();
    assert_middleware_processed(&processed_ctx, "test1");
}

#[tokio::test]
async fn test_middleware_stack_multiple_middleware() {
    let mut stack = MiddlewareStack::new();

    // Add multiple middleware
    stack.add(TestMiddleware::new("first"));
    stack.add(TestMiddleware::new("second"));
    stack.add(TestMiddleware::new("third"));

    let request = create_test_request();
    let ctx = create_test_context();
    let (_, processed_ctx) = stack.process_request(request, ctx).await.unwrap();

    // Verify all processed
    assert_middleware_processed(&processed_ctx, "first");
    assert_middleware_processed(&processed_ctx, "second");
    assert_middleware_processed(&processed_ctx, "third");
}

// ============================================================================
// Error Handling and Edge Cases
// ============================================================================

#[tokio::test]
async fn test_middleware_failure_handling() {
    // Test with recovery disabled to see actual failures
    let config = StackConfig {
        enable_recovery: false,
        ..StackConfig::default()
    };
    let mut stack = MiddlewareStack::with_config(config);
    stack.add(TestMiddleware::new("good1"));
    stack.add(TestMiddleware::new("failing").with_failure());
    stack.add(TestMiddleware::new("good2"));

    let request = create_test_request();
    let ctx = create_test_context();
    let result = stack.process_request(request, ctx).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Test middleware failing failed")
    );
}

#[tokio::test]
async fn test_middleware_timeout_handling() {
    let config = StackConfig {
        timeout_ms: 50,
        enable_recovery: false, // Disable recovery to see timeout errors
        ..StackConfig::default()
    };
    let mut stack = MiddlewareStack::with_config(config);

    // Add slow middleware that should timeout
    stack.add(TestMiddleware::new("slow").with_delay(100));

    let request = create_test_request();
    let ctx = create_test_context();

    let result = stack.process_request(request, ctx).await;

    // Should fail due to either timeout or middleware error
    assert!(result.is_err());
}

// ============================================================================
// Request/Response Mutation Tests
// ============================================================================

#[tokio::test]
async fn test_middleware_request_mutation() {
    struct RequestMutator;

    #[async_trait]
    impl Middleware for RequestMutator {
        async fn process_request(
            &self,
            request: &mut JsonRpcRequest,
            _ctx: &mut RequestContext,
        ) -> ServerResult<()> {
            request.method = "mutated/method".to_string();
            Ok(())
        }

        async fn process_response(
            &self,
            _response: &mut JsonRpcResponse,
            _ctx: &RequestContext,
        ) -> ServerResult<()> {
            Ok(())
        }

        fn name(&self) -> &str {
            "request_mutator"
        }
    }

    let mut stack = MiddlewareStack::new();
    stack.add(RequestMutator);

    let request = create_test_request();
    let original_method = request.method.clone();
    let ctx = create_test_context();

    let (mutated_request, _) = stack.process_request(request, ctx).await.unwrap();
    assert_ne!(mutated_request.method, original_method);
    assert_eq!(mutated_request.method, "mutated/method");
}

#[tokio::test]
async fn test_middleware_response_processing() {
    let mut stack = MiddlewareStack::new();
    stack.add(TestMiddleware::new("response_processor"));

    let response = create_test_response();
    let ctx = create_test_context();

    let result = stack.process_response(response, &ctx).await;
    assert!(result.is_ok());

    // Test failure case with recovery disabled
    let config = StackConfig {
        enable_recovery: false,
        ..StackConfig::default()
    };
    let mut failing_stack = MiddlewareStack::with_config(config);
    failing_stack.add(TestMiddleware::new("failing_processor").with_failure());
    let response2 = create_test_response();
    let result = failing_stack.process_response(response2, &ctx).await;
    assert!(result.is_err());
}

// ============================================================================
// Integration and Performance Tests
// ============================================================================

#[tokio::test]
async fn test_middleware_stack_performance() {
    let mut stack = MiddlewareStack::new();

    // Add many lightweight middleware
    for i in 0..50 {
        stack.add(TestMiddleware::new(&format!("middleware_{i}")));
    }

    let request = create_test_request();
    let ctx = create_test_context();

    let result = stack.process_request(request, ctx).await;

    assert!(result.is_ok());
    // Performance test - should handle 50 middleware efficiently
}

#[tokio::test]
async fn test_middleware_metadata_propagation() {
    let mut stack = MiddlewareStack::new();
    stack.add(TestMiddleware::new("metadata1"));
    stack.add(TestMiddleware::new("metadata2"));

    let request = create_test_request();
    let ctx = create_test_context();

    let (_, processed_ctx) = stack.process_request(request, ctx).await.unwrap();

    // Should have original metadata plus middleware additions
    assert!(processed_ctx.metadata.contains_key("test_key"));
    assert_middleware_processed(&processed_ctx, "metadata1");
    assert_middleware_processed(&processed_ctx, "metadata2");
    assert!(processed_ctx.metadata.contains_key("correlation_id"));
}

// ============================================================================
// Property-based Tests
// ============================================================================

#[test]
fn test_middleware_configuration_properties() {
    test_config_properties::<StackConfig>();
}
