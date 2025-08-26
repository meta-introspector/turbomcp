//! Comprehensive tests for the session module to improve coverage
//!
//! This test suite targets the session module which provides session management,
//! client tracking, request analytics, and cleanup functionality for MCP applications.

use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use tokio::time::sleep;
use turbomcp_core::context::RequestInfo;
use turbomcp_core::session::*;

// ============================================================================
// SessionConfig Tests
// ============================================================================

#[test]
fn test_session_config_default() {
    let config = SessionConfig::default();

    assert_eq!(config.max_sessions, 1000);
    assert_eq!(config.session_timeout, Duration::hours(24));
    assert_eq!(config.max_request_history, 1000);
    assert!(config.max_requests_per_session.is_none());
    assert_eq!(config.cleanup_interval, StdDuration::from_secs(300));
    assert!(config.enable_analytics);
}

#[test]
fn test_session_config_custom() {
    let config = SessionConfig {
        max_sessions: 500,
        session_timeout: Duration::hours(12),
        max_request_history: 2000,
        max_requests_per_session: Some(100),
        cleanup_interval: StdDuration::from_secs(60),
        enable_analytics: false,
    };

    assert_eq!(config.max_sessions, 500);
    assert_eq!(config.session_timeout, Duration::hours(12));
    assert_eq!(config.max_request_history, 2000);
    assert_eq!(config.max_requests_per_session, Some(100));
    assert_eq!(config.cleanup_interval, StdDuration::from_secs(60));
    assert!(!config.enable_analytics);
}

#[test]
fn test_session_config_serialization() {
    let config = SessionConfig::default();

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: SessionConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.max_sessions, deserialized.max_sessions);
    assert_eq!(config.session_timeout, deserialized.session_timeout);
    assert_eq!(config.max_request_history, deserialized.max_request_history);
    assert_eq!(
        config.max_requests_per_session,
        deserialized.max_requests_per_session
    );
    assert_eq!(config.cleanup_interval, deserialized.cleanup_interval);
    assert_eq!(config.enable_analytics, deserialized.enable_analytics);
}

#[test]
fn test_session_config_debug_clone() {
    let config = SessionConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("SessionConfig"));
    assert!(debug_str.contains("max_sessions"));

    let cloned = config.clone();
    assert_eq!(config.max_sessions, cloned.max_sessions);
    assert_eq!(config.enable_analytics, cloned.enable_analytics);
}

// ============================================================================
// SessionEventType Tests
// ============================================================================

#[test]
fn test_session_event_type_variants() {
    let events = vec![
        SessionEventType::Created,
        SessionEventType::Authenticated,
        SessionEventType::Updated,
        SessionEventType::Expired,
        SessionEventType::Terminated,
    ];

    for event in events {
        let debug_str = format!("{event:?}");
        assert!(!debug_str.is_empty());

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SessionEventType = serde_json::from_str(&json).unwrap();

        // Use format comparison since PartialEq might not be derived
        assert_eq!(format!("{event:?}"), format!("{:?}", deserialized));
    }
}

#[test]
fn test_session_event_serialization() {
    let event = SessionEvent {
        timestamp: Utc::now(),
        client_id: "client-123".to_string(),
        event_type: SessionEventType::Created,
        metadata: {
            let mut map = HashMap::new();
            map.insert("transport".to_string(), serde_json::json!("websocket"));
            map
        },
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: SessionEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.client_id, deserialized.client_id);
    assert_eq!(event.metadata, deserialized.metadata);
    // Use debug comparison for event_type since PartialEq might not be derived
    assert_eq!(
        format!("{:?}", event.event_type),
        format!("{:?}", deserialized.event_type)
    );
}

#[test]
fn test_session_event_debug_clone() {
    let event = SessionEvent {
        timestamp: Utc::now(),
        client_id: "client-456".to_string(),
        event_type: SessionEventType::Authenticated,
        metadata: HashMap::new(),
    };

    let debug_str = format!("{event:?}");
    assert!(debug_str.contains("SessionEvent"));
    assert!(debug_str.contains("client-456"));

    let cloned = event.clone();
    assert_eq!(event.client_id, cloned.client_id);
    assert_eq!(event.metadata, cloned.metadata);
}

// ============================================================================
// SessionAnalytics Tests
// ============================================================================

#[test]
fn test_session_analytics_debug_clone() {
    let analytics = SessionAnalytics {
        total_sessions: 10,
        active_sessions: 5,
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        avg_session_duration: Duration::minutes(30),
        top_clients: vec![("client-1".to_string(), 50), ("client-2".to_string(), 30)],
        top_methods: vec![("method-1".to_string(), 40), ("method-2".to_string(), 35)],
        requests_per_minute: 2.5,
    };

    let debug_str = format!("{analytics:?}");
    assert!(debug_str.contains("SessionAnalytics"));
    assert!(debug_str.contains("total_sessions"));

    let cloned = analytics.clone();
    assert_eq!(analytics.total_sessions, cloned.total_sessions);
    assert_eq!(analytics.active_sessions, cloned.active_sessions);
    assert_eq!(analytics.total_requests, cloned.total_requests);
    assert_eq!(analytics.top_clients, cloned.top_clients);
    assert_eq!(analytics.top_methods, cloned.top_methods);
}

#[test]
fn test_session_analytics_serialization() {
    let analytics = SessionAnalytics {
        total_sessions: 25,
        active_sessions: 15,
        total_requests: 500,
        successful_requests: 475,
        failed_requests: 25,
        avg_session_duration: Duration::hours(2),
        top_clients: vec![("client-a".to_string(), 100), ("client-b".to_string(), 80)],
        top_methods: vec![
            ("list_tools".to_string(), 150),
            ("call_tool".to_string(), 120),
        ],
        requests_per_minute: 5.2,
    };

    let json = serde_json::to_string(&analytics).unwrap();
    let deserialized: SessionAnalytics = serde_json::from_str(&json).unwrap();

    assert_eq!(analytics.total_sessions, deserialized.total_sessions);
    assert_eq!(analytics.active_sessions, deserialized.active_sessions);
    assert_eq!(analytics.total_requests, deserialized.total_requests);
    assert_eq!(
        analytics.successful_requests,
        deserialized.successful_requests
    );
    assert_eq!(analytics.failed_requests, deserialized.failed_requests);
    assert_eq!(
        analytics.avg_session_duration,
        deserialized.avg_session_duration
    );
    assert_eq!(analytics.top_clients, deserialized.top_clients);
    assert_eq!(analytics.top_methods, deserialized.top_methods);
    assert!((analytics.requests_per_minute - deserialized.requests_per_minute).abs() < 0.001);
}

// ============================================================================
// SessionManager Core Tests
// ============================================================================

#[tokio::test]
async fn test_session_manager_new() {
    let config = SessionConfig::default();
    let manager = SessionManager::new(config.clone());

    let debug_str = format!("{manager:?}");
    assert!(debug_str.contains("SessionManager"));

    // Test that it starts with empty state
    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_sessions, 0);
    assert_eq!(analytics.active_sessions, 0);
    assert_eq!(analytics.total_requests, 0);
}

#[tokio::test]
async fn test_session_manager_default() {
    let manager = SessionManager::default();
    let analytics = manager.get_analytics();

    assert_eq!(analytics.total_sessions, 0);
    assert_eq!(analytics.active_sessions, 0);
}

#[tokio::test]
async fn test_get_or_create_session_new() {
    let manager = SessionManager::new(SessionConfig::default());

    let session = manager.get_or_create_session("client-new".to_string(), "http".to_string());

    assert_eq!(session.client_id, "client-new");
    assert_eq!(session.transport_type, "http");
    assert!(!session.authenticated);
    assert_eq!(session.request_count, 0);

    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_sessions, 1);
    assert_eq!(analytics.active_sessions, 1);
}

#[tokio::test]
async fn test_get_or_create_session_existing() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create first session
    let session1 =
        manager.get_or_create_session("client-existing".to_string(), "websocket".to_string());
    assert_eq!(session1.client_id, "client-existing");

    // Get the same session again
    let session2 = manager.get_or_create_session(
        "client-existing".to_string(),
        "different-transport".to_string(),
    );
    assert_eq!(session2.client_id, "client-existing");
    assert_eq!(session2.transport_type, "websocket"); // Should keep original transport

    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_sessions, 1); // Still only one session
    assert_eq!(analytics.active_sessions, 1);
}

#[tokio::test]
async fn test_get_session() {
    let manager = SessionManager::new(SessionConfig::default());

    // Should return None for non-existent session
    assert!(manager.get_session("non-existent").is_none());

    // Create a session
    let _ = manager.get_or_create_session("client-get".to_string(), "stdio".to_string());

    // Should return the session
    let session = manager.get_session("client-get");
    assert!(session.is_some());
    assert_eq!(session.unwrap().client_id, "client-get");
}

#[tokio::test]
async fn test_get_active_sessions() {
    let manager = SessionManager::new(SessionConfig::default());

    // Initially empty
    let active = manager.get_active_sessions();
    assert_eq!(active.len(), 0);

    // Create multiple sessions
    let _ = manager.get_or_create_session("client-1".to_string(), "http".to_string());
    let _ = manager.get_or_create_session("client-2".to_string(), "websocket".to_string());
    let _ = manager.get_or_create_session("client-3".to_string(), "stdio".to_string());

    let active = manager.get_active_sessions();
    assert_eq!(active.len(), 3);

    let client_ids: Vec<String> = active.iter().map(|s| s.client_id.clone()).collect();
    assert!(client_ids.contains(&"client-1".to_string()));
    assert!(client_ids.contains(&"client-2".to_string()));
    assert!(client_ids.contains(&"client-3".to_string()));
}

#[tokio::test]
async fn test_client_extractor() {
    let manager = SessionManager::new(SessionConfig::default());
    let extractor = manager.client_extractor();

    // Should be able to use the extractor
    extractor.register_token("test-token".to_string(), "test-client".to_string());
    let tokens = extractor.list_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(tokens.contains(&("test-token".to_string(), "test-client".to_string())));
}

// ============================================================================
// Session Activity and Authentication Tests
// ============================================================================

#[tokio::test]
async fn test_update_client_activity() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create a session
    let session = manager.get_or_create_session("client-activity".to_string(), "http".to_string());
    let initial_count = session.request_count;

    // Update activity
    manager.update_client_activity("client-activity");

    // Get updated session and verify activity was incremented
    let updated_session = manager.get_session("client-activity").unwrap();
    assert_eq!(updated_session.request_count, initial_count + 1);

    // Update activity for non-existent client (should not panic)
    manager.update_client_activity("non-existent");
}

#[tokio::test]
async fn test_update_client_activity_with_request_cap() {
    let config = SessionConfig {
        max_requests_per_session: Some(2), // Very low cap for testing
        ..SessionConfig::default()
    };
    let manager = SessionManager::new(config);

    // Create a session
    let _ = manager.get_or_create_session("capped-client".to_string(), "http".to_string());

    // Update activity twice (within cap)
    manager.update_client_activity("capped-client");
    manager.update_client_activity("capped-client");
    assert!(manager.get_session("capped-client").is_some());

    // Update activity once more (exceeds cap)
    manager.update_client_activity("capped-client");

    // Session should be terminated
    assert!(manager.get_session("capped-client").is_none());
}

#[tokio::test]
async fn test_authenticate_client_success() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create a session
    let _ = manager.get_or_create_session("auth-client".to_string(), "websocket".to_string());

    // Authenticate with token
    let success = manager.authenticate_client(
        "auth-client",
        Some("Authenticated Client".to_string()),
        Some("auth-token-123".to_string()),
    );

    assert!(success);

    // Verify session is authenticated
    let session = manager.get_session("auth-client").unwrap();
    assert!(session.authenticated);
    assert_eq!(
        session.client_name,
        Some("Authenticated Client".to_string())
    );

    // Verify token was registered
    let extractor = manager.client_extractor();
    let tokens = extractor.list_tokens();
    assert!(tokens.contains(&("auth-token-123".to_string(), "auth-client".to_string())));
}

#[tokio::test]
async fn test_authenticate_client_without_token() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create a session
    let _ = manager.get_or_create_session("auth-no-token".to_string(), "stdio".to_string());

    // Authenticate without token
    let success = manager.authenticate_client(
        "auth-no-token",
        Some("Client Without Token".to_string()),
        None,
    );

    assert!(success);

    // Verify session is authenticated
    let session = manager.get_session("auth-no-token").unwrap();
    assert!(session.authenticated);
    assert_eq!(
        session.client_name,
        Some("Client Without Token".to_string())
    );

    // Verify no token was registered
    let extractor = manager.client_extractor();
    let tokens = extractor.list_tokens();
    assert!(tokens.is_empty());
}

#[tokio::test]
async fn test_authenticate_client_without_name() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create a session
    let _ = manager.get_or_create_session("auth-no-name".to_string(), "http".to_string());

    // Authenticate without name
    let success = manager.authenticate_client("auth-no-name", None, Some("token-456".to_string()));

    assert!(success);

    // Verify session is authenticated but has no name
    let session = manager.get_session("auth-no-name").unwrap();
    assert!(session.authenticated);
    assert!(session.client_name.is_none());
}

#[tokio::test]
async fn test_authenticate_client_non_existent() {
    let manager = SessionManager::new(SessionConfig::default());

    // Try to authenticate non-existent client
    let success = manager.authenticate_client(
        "non-existent",
        Some("Name".to_string()),
        Some("token".to_string()),
    );

    assert!(!success);
}

// ============================================================================
// Request Recording and Analytics Tests
// ============================================================================

#[tokio::test]
async fn test_record_request_success() {
    let manager = SessionManager::new(SessionConfig::default());

    let request = RequestInfo::new(
        "analytics-client".to_string(),
        "test_method".to_string(),
        serde_json::json!({"param": "value"}),
    )
    .complete_success(150);

    manager.record_request(request);

    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_requests, 1);
    assert_eq!(analytics.successful_requests, 1);
    assert_eq!(analytics.failed_requests, 0);
}

#[tokio::test]
async fn test_record_request_failure() {
    let manager = SessionManager::new(SessionConfig::default());

    let request = RequestInfo::new(
        "error-client".to_string(),
        "failing_method".to_string(),
        serde_json::json!({}),
    )
    .complete_error(75, "Something went wrong".to_string());

    manager.record_request(request);

    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_requests, 1);
    assert_eq!(analytics.successful_requests, 0);
    assert_eq!(analytics.failed_requests, 1);
}

#[tokio::test]
async fn test_record_request_analytics_disabled() {
    let config = SessionConfig {
        enable_analytics: false,
        ..Default::default()
    };
    let manager = SessionManager::new(config);

    let request = RequestInfo::new(
        "disabled-client".to_string(),
        "method".to_string(),
        serde_json::json!({}),
    )
    .complete_success(100);

    manager.record_request(request);

    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_requests, 0); // Should not record when disabled
}

#[tokio::test]
async fn test_record_request_parameter_sanitization() {
    let manager = SessionManager::new(SessionConfig::default());

    let request = RequestInfo::new(
        "sanitize-client".to_string(),
        "sensitive_method".to_string(),
        serde_json::json!({
            "username": "testuser",
            "password": "secret123",
            "api_key": "key456",
            "token": "token789",
            "secret": "topsecret",
            "auth": "bearer xyz",
            "normal_data": "public_info"
        }),
    )
    .complete_success(100);

    manager.record_request(request);

    let history = manager.get_request_history(Some(1));
    assert_eq!(history.len(), 1);

    let params = &history[0].parameters;
    let obj = params.as_object().unwrap();

    // Sensitive fields should be redacted
    assert_eq!(obj["password"], serde_json::json!("[REDACTED]"));
    assert_eq!(obj["api_key"], serde_json::json!("[REDACTED]"));
    assert_eq!(obj["token"], serde_json::json!("[REDACTED]"));
    assert_eq!(obj["secret"], serde_json::json!("[REDACTED]"));
    assert_eq!(obj["auth"], serde_json::json!("[REDACTED]"));

    // Non-sensitive fields should remain
    assert_eq!(obj["username"], serde_json::json!("testuser"));
    assert_eq!(obj["normal_data"], serde_json::json!("public_info"));
}

#[tokio::test]
async fn test_get_request_history() {
    let manager = SessionManager::new(SessionConfig::default());

    // Record multiple requests
    for i in 0..5 {
        let request = RequestInfo::new(
            format!("client-{i}"),
            format!("method_{i}"),
            serde_json::json!({"index": i}),
        )
        .complete_success(100 + i as u64);

        manager.record_request(request);
    }

    // Get all history
    let all_history = manager.get_request_history(None);
    assert_eq!(all_history.len(), 5);

    // History should be in reverse order (most recent first)
    assert_eq!(all_history[0].method_name, "method_4");
    assert_eq!(all_history[4].method_name, "method_0");

    // Get limited history
    let limited_history = manager.get_request_history(Some(3));
    assert_eq!(limited_history.len(), 3);
    assert_eq!(limited_history[0].method_name, "method_4");
    assert_eq!(limited_history[2].method_name, "method_2");
}

#[tokio::test]
async fn test_request_history_capacity() {
    let config = SessionConfig {
        max_request_history: 3, // Very small for testing
        ..Default::default()
    };
    let manager = SessionManager::new(config);

    // Record more requests than capacity
    for i in 0..5 {
        let request = RequestInfo::new(
            "capacity-client".to_string(),
            format!("method_{i}"),
            serde_json::json!({}),
        )
        .complete_success(100);

        manager.record_request(request);
    }

    // Should only keep the last 3 requests
    let history = manager.get_request_history(None);
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].method_name, "method_4"); // Most recent
    assert_eq!(history[1].method_name, "method_3");
    assert_eq!(history[2].method_name, "method_2"); // Oldest kept
}

// ============================================================================
// Session Termination Tests
// ============================================================================

#[tokio::test]
async fn test_terminate_session_success() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create a session
    let _ = manager.get_or_create_session("terminate-me".to_string(), "http".to_string());
    assert!(manager.get_session("terminate-me").is_some());

    // Terminate the session
    let terminated = manager.terminate_session("terminate-me");
    assert!(terminated);

    // Session should be gone
    assert!(manager.get_session("terminate-me").is_none());

    let analytics = manager.get_analytics();
    assert_eq!(analytics.active_sessions, 0);
}

#[tokio::test]
async fn test_terminate_session_non_existent() {
    let manager = SessionManager::new(SessionConfig::default());

    // Try to terminate non-existent session
    let terminated = manager.terminate_session("does-not-exist");
    assert!(!terminated);
}

// ============================================================================
// Session Events Tests
// ============================================================================

#[tokio::test]
async fn test_get_session_events() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create and authenticate a session (generates events)
    let _ = manager.get_or_create_session("events-client".to_string(), "websocket".to_string());
    let _ = manager.authenticate_client("events-client", Some("Events Client".to_string()), None);
    let _ = manager.terminate_session("events-client");

    // Get events
    let events = manager.get_session_events(None);
    assert!(events.len() >= 3); // Created, Authenticated, Terminated

    // Events should be in reverse order (most recent first)
    let client_ids: Vec<&String> = events.iter().map(|e| &e.client_id).collect();
    assert!(client_ids.contains(&&"events-client".to_string()));

    // Test limited events
    let limited_events = manager.get_session_events(Some(2));
    assert!(limited_events.len() <= 2);
}

// ============================================================================
// Analytics and Statistics Tests
// ============================================================================

#[tokio::test]
async fn test_analytics_calculation() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create multiple sessions
    let _ = manager.get_or_create_session("analytics-1".to_string(), "http".to_string());
    let _ = manager.get_or_create_session("analytics-2".to_string(), "websocket".to_string());

    // Record requests from different clients and methods
    let requests = vec![
        ("analytics-1", "method_a", true),
        ("analytics-1", "method_a", true),
        ("analytics-1", "method_b", false),
        ("analytics-2", "method_a", true),
        ("analytics-2", "method_c", true),
        ("analytics-1", "method_b", true),
    ];

    for (client, method, success) in requests {
        let mut request = RequestInfo::new(
            client.to_string(),
            method.to_string(),
            serde_json::json!({}),
        );

        if success {
            request = request.complete_success(100);
        } else {
            request = request.complete_error(50, "Error".to_string());
        }

        manager.record_request(request);
    }

    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_sessions, 2);
    assert_eq!(analytics.active_sessions, 2);
    assert_eq!(analytics.total_requests, 6);
    assert_eq!(analytics.successful_requests, 5);
    assert_eq!(analytics.failed_requests, 1);

    // Check top clients (sorted by request count)
    assert!(!analytics.top_clients.is_empty());
    let top_client = &analytics.top_clients[0];
    assert_eq!(top_client.0, "analytics-1"); // Should have most requests (4)
    assert_eq!(top_client.1, 4);

    // Check top methods
    assert!(!analytics.top_methods.is_empty());
    let top_method = &analytics.top_methods[0];
    assert_eq!(top_method.0, "method_a"); // Should have most requests (3)
    assert_eq!(top_method.1, 3);
}

#[tokio::test]
async fn test_analytics_empty_state() {
    let manager = SessionManager::new(SessionConfig::default());

    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_sessions, 0);
    assert_eq!(analytics.active_sessions, 0);
    assert_eq!(analytics.total_requests, 0);
    assert_eq!(analytics.successful_requests, 0);
    assert_eq!(analytics.failed_requests, 0);
    assert_eq!(analytics.avg_session_duration, Duration::zero());
    assert!(analytics.top_clients.is_empty());
    assert!(analytics.top_methods.is_empty());
    assert_eq!(analytics.requests_per_minute, 0.0);
}

#[tokio::test]
async fn test_analytics_requests_per_minute() {
    let manager = SessionManager::new(SessionConfig::default());

    // Record a request (within the last hour)
    let request = RequestInfo::new(
        "rpm-client".to_string(),
        "rpm_method".to_string(),
        serde_json::json!({}),
    )
    .complete_success(100);

    manager.record_request(request);

    let analytics = manager.get_analytics();
    assert!(analytics.requests_per_minute > 0.0);
    assert!(analytics.requests_per_minute <= 1.0 / 60.0); // At most 1 request in the last hour
}

// ============================================================================
// Session Manager Start and Cleanup Tests
// ============================================================================

#[tokio::test]
async fn test_session_manager_start() {
    let manager = SessionManager::new(SessionConfig::default());

    // Start should not fail
    manager.start();

    // Starting again should be safe
    manager.start();
}

#[tokio::test]
async fn test_capacity_enforcement() {
    let config = SessionConfig {
        max_sessions: 2, // Very small for testing
        ..Default::default()
    };
    let manager = SessionManager::new(config);

    // Create sessions up to capacity
    let _ = manager.get_or_create_session("capacity-1".to_string(), "http".to_string());
    let _ = manager.get_or_create_session("capacity-2".to_string(), "websocket".to_string());

    // Both sessions should exist
    assert!(manager.get_session("capacity-1").is_some());
    assert!(manager.get_session("capacity-2").is_some());
    assert_eq!(manager.get_active_sessions().len(), 2);

    // Add activity to capacity-2 to make it more recent
    sleep(StdDuration::from_millis(10)).await; // Small delay to ensure time difference
    manager.update_client_activity("capacity-2");

    // Create one more session (should trigger eviction)
    let _ = manager.get_or_create_session("capacity-3".to_string(), "stdio".to_string());

    // Should still have only 2 sessions total
    assert_eq!(manager.get_active_sessions().len(), 2);

    // capacity-1 should be evicted (least recent activity)
    assert!(manager.get_session("capacity-1").is_none());
    assert!(manager.get_session("capacity-2").is_some());
    assert!(manager.get_session("capacity-3").is_some());
}

#[tokio::test]
async fn test_capacity_enforcement_empty_sessions() {
    let config = SessionConfig {
        max_sessions: 5,
        ..Default::default()
    };
    let manager = SessionManager::new(config);

    // Creating a session when under capacity should not trigger any eviction
    let _ = manager.get_or_create_session("under-capacity".to_string(), "http".to_string());
    assert_eq!(manager.get_active_sessions().len(), 1);
}

// ============================================================================
// Integration and Complex Scenarios Tests
// ============================================================================

#[tokio::test]
async fn test_full_session_lifecycle() {
    let manager = SessionManager::new(SessionConfig::default());

    // 1. Create session
    let session = manager.get_or_create_session("lifecycle".to_string(), "websocket".to_string());
    assert_eq!(session.client_id, "lifecycle");
    assert!(!session.authenticated);

    // 2. Authenticate session
    let success = manager.authenticate_client(
        "lifecycle",
        Some("Lifecycle Client".to_string()),
        Some("lifecycle-token".to_string()),
    );
    assert!(success);

    // 3. Record multiple requests
    for i in 0..3 {
        let request = RequestInfo::new(
            "lifecycle".to_string(),
            format!("operation_{i}"),
            serde_json::json!({"step": i}),
        )
        .complete_success(100 + i as u64 * 10);

        manager.record_request(request);
    }

    // 4. Check analytics
    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_sessions, 1);
    assert_eq!(analytics.active_sessions, 1);
    assert_eq!(analytics.total_requests, 3);
    assert_eq!(analytics.successful_requests, 3);

    // 5. Terminate session
    let terminated = manager.terminate_session("lifecycle");
    assert!(terminated);

    // 6. Verify session is gone but analytics are preserved
    assert!(manager.get_session("lifecycle").is_none());
    let final_analytics = manager.get_analytics();
    assert_eq!(final_analytics.total_sessions, 1); // Still counted
    assert_eq!(final_analytics.active_sessions, 0); // But not active
    assert_eq!(final_analytics.total_requests, 3); // Preserved
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    let manager = std::sync::Arc::new(SessionManager::new(SessionConfig::default()));

    // Spawn multiple tasks creating sessions concurrently
    let mut handles = Vec::new();

    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let client_id = format!("concurrent-{i}");

            // Create session
            let _ = manager_clone.get_or_create_session(client_id.clone(), "http".to_string());

            // Authenticate
            let _ = manager_clone.authenticate_client(
                &client_id,
                Some(format!("Client {i}")),
                Some(format!("token-{i}")),
            );

            // Record some requests
            for j in 0..3 {
                let request = RequestInfo::new(
                    client_id.clone(),
                    format!("method_{i}_{j}"),
                    serde_json::json!({"i": i, "j": j}),
                )
                .complete_success(100);

                manager_clone.record_request(request);
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all sessions were created and requests recorded
    let analytics = manager.get_analytics();
    assert_eq!(analytics.total_sessions, 10);
    assert_eq!(analytics.active_sessions, 10);
    assert_eq!(analytics.total_requests, 30); // 10 clients * 3 requests each
    assert_eq!(analytics.successful_requests, 30);

    let active_sessions = manager.get_active_sessions();
    assert_eq!(active_sessions.len(), 10);

    // All sessions should be authenticated
    for session in &active_sessions {
        assert!(session.authenticated);
        assert!(session.client_name.is_some());
    }
}

#[tokio::test]
async fn test_session_event_history_capacity() {
    let manager = SessionManager::new(SessionConfig::default());

    // Create and terminate many sessions to generate events (more than 1000 to test capacity)
    for i in 0..1005 {
        let client_id = format!("event-history-{i}");
        let _ = manager.get_or_create_session(client_id.clone(), "http".to_string());
        let _ = manager.terminate_session(&client_id);
    }

    // Should not exceed capacity
    let events = manager.get_session_events(None);
    assert!(events.len() <= 1000); // Should be capped at internal limit

    // Most recent events should be preserved
    let recent_events = manager.get_session_events(Some(10));
    assert_eq!(recent_events.len(), 10);
}

#[tokio::test]
async fn test_sanitization_edge_cases_via_record_request() {
    let manager = SessionManager::new(SessionConfig::default());

    // Test non-object parameters (should be recorded as-is)
    let array_request = RequestInfo::new(
        "array-client".to_string(),
        "array_method".to_string(),
        serde_json::json!(["item1", "item2"]),
    )
    .complete_success(100);

    manager.record_request(array_request);

    let string_request = RequestInfo::new(
        "string-client".to_string(),
        "string_method".to_string(),
        serde_json::json!("just a string"),
    )
    .complete_success(100);

    manager.record_request(string_request);

    let number_request = RequestInfo::new(
        "number-client".to_string(),
        "number_method".to_string(),
        serde_json::json!(42),
    )
    .complete_success(100);

    manager.record_request(number_request);

    // Test object without sensitive fields
    let safe_request = RequestInfo::new(
        "safe-client".to_string(),
        "safe_method".to_string(),
        serde_json::json!({
            "username": "user",
            "action": "list",
            "limit": 10
        }),
    )
    .complete_success(100);

    manager.record_request(safe_request);

    // Verify requests were recorded
    let history = manager.get_request_history(None);
    assert_eq!(history.len(), 4);

    // Find and verify each request type
    let array_req = history
        .iter()
        .find(|r| r.method_name == "array_method")
        .unwrap();
    assert_eq!(array_req.parameters, serde_json::json!(["item1", "item2"]));

    let string_req = history
        .iter()
        .find(|r| r.method_name == "string_method")
        .unwrap();
    assert_eq!(string_req.parameters, serde_json::json!("just a string"));

    let number_req = history
        .iter()
        .find(|r| r.method_name == "number_method")
        .unwrap();
    assert_eq!(number_req.parameters, serde_json::json!(42));

    let safe_req = history
        .iter()
        .find(|r| r.method_name == "safe_method")
        .unwrap();
    let safe_obj = safe_req.parameters.as_object().unwrap();
    assert_eq!(safe_obj["username"], serde_json::json!("user"));
    assert_eq!(safe_obj["action"], serde_json::json!("list"));
    assert_eq!(safe_obj["limit"], serde_json::json!(10));
}
