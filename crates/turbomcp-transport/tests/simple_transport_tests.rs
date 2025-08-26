//! Simple transport functionality tests
//! Focus on basic types and functionality that compiles

use std::collections::HashMap;

use turbomcp_transport::core::*;

// ============================================================================
// Basic Type Tests
// ============================================================================

#[test]
fn test_transport_type_creation() {
    let stdio = TransportType::Stdio;
    let http = TransportType::Http;
    let websocket = TransportType::WebSocket;
    let tcp = TransportType::Tcp;
    let unix = TransportType::Unix;

    assert_eq!(stdio, TransportType::Stdio);
    assert_eq!(http, TransportType::Http);
    assert_eq!(websocket, TransportType::WebSocket);
    assert_eq!(tcp, TransportType::Tcp);
    assert_eq!(unix, TransportType::Unix);
}

#[test]
fn test_transport_type_debug() {
    let transport_type = TransportType::Stdio;
    let debug_str = format!("{transport_type:?}");
    assert!(debug_str.contains("Stdio"));
}

#[test]
fn test_transport_type_clone() {
    let original = TransportType::Http;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn test_transport_type_serialization() {
    let transport_type = TransportType::Stdio;
    let serialized = serde_json::to_string(&transport_type).unwrap();
    let deserialized: TransportType = serde_json::from_str(&serialized).unwrap();
    assert_eq!(transport_type, deserialized);
}

// ============================================================================
// TransportState Tests
// ============================================================================

#[test]
fn test_transport_state_variants() {
    let disconnected = TransportState::Disconnected;
    let connecting = TransportState::Connecting;
    let connected = TransportState::Connected;
    let disconnecting = TransportState::Disconnecting;
    let failed = TransportState::Failed {
        reason: "test error".to_string(),
    };

    assert_eq!(disconnected, TransportState::Disconnected);
    assert_eq!(connecting, TransportState::Connecting);
    assert_eq!(connected, TransportState::Connected);
    assert_eq!(disconnecting, TransportState::Disconnecting);

    match failed {
        TransportState::Failed { reason } => {
            assert_eq!(reason, "test error");
        }
        _ => panic!("Expected failed state"),
    }
}

#[test]
fn test_transport_state_debug() {
    let state = TransportState::Connected;
    let debug_str = format!("{state:?}");
    assert!(debug_str.contains("Connected"));
}

#[test]
fn test_transport_state_clone() {
    let original = TransportState::Failed {
        reason: "network error".to_string(),
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_transport_state_serialization() {
    let state = TransportState::Connecting;
    let serialized = serde_json::to_string(&state).unwrap();
    let deserialized: TransportState = serde_json::from_str(&serialized).unwrap();
    assert_eq!(state, deserialized);
}

// ============================================================================
// TransportError Tests
// ============================================================================

#[test]
fn test_transport_error_creation() {
    let error = TransportError::ConnectionFailed("test".to_string());
    let error_str = format!("{error}");
    assert!(error_str.contains("Connection failed"));
    assert!(error_str.contains("test"));
}

#[test]
fn test_transport_error_variants() {
    let errors = vec![
        TransportError::ConnectionFailed("conn failed".to_string()),
        TransportError::ConnectionLost("conn lost".to_string()),
        TransportError::SendFailed("send failed".to_string()),
        TransportError::ReceiveFailed("receive failed".to_string()),
        TransportError::Timeout,
        TransportError::Internal("internal error".to_string()),
    ];

    for error in errors {
        let error_str = format!("{error}");
        assert!(!error_str.is_empty());

        let debug_str = format!("{error:?}");
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_transport_error_clone() {
    let original = TransportError::Timeout;
    let cloned = original.clone();

    match (&original, &cloned) {
        (TransportError::Timeout, TransportError::Timeout) => {
            // Success - both are timeout errors
        }
        _ => panic!("Clone did not preserve error type"),
    }
}

// ============================================================================
// TransportCapabilities Tests
// ============================================================================

#[test]
fn test_transport_capabilities_default() {
    let capabilities = TransportCapabilities::default();

    // Just test that default can be created and has expected structure
    let _max_size = capabilities.max_message_size; // May or may not be Some
    let _compression = capabilities.supports_compression; // May be true or false
    let _streaming = capabilities.supports_streaming;
    let _bidirectional = capabilities.supports_bidirectional;
    let _multiplexing = capabilities.supports_multiplexing;
    let _algorithms = &capabilities.compression_algorithms;
    let _custom = &capabilities.custom;

    // Test that debug formatting works
    let debug_str = format!("{capabilities:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn test_transport_capabilities_creation() {
    let mut custom = HashMap::new();
    custom.insert("feature".to_string(), serde_json::json!("enabled"));

    let capabilities = TransportCapabilities {
        max_message_size: Some(1024),
        supports_compression: true,
        supports_streaming: true,
        supports_bidirectional: false,
        supports_multiplexing: false,
        compression_algorithms: vec!["gzip".to_string()],
        custom,
    };

    assert_eq!(capabilities.max_message_size, Some(1024));
    assert!(capabilities.supports_compression);
    assert!(capabilities.supports_streaming);
    assert!(!capabilities.supports_bidirectional);
    assert_eq!(capabilities.compression_algorithms.len(), 1);
    assert_eq!(capabilities.custom.len(), 1);
}

#[test]
fn test_transport_capabilities_serialization() {
    let capabilities = TransportCapabilities {
        max_message_size: Some(2048),
        supports_compression: true,
        ..Default::default()
    };

    let serialized = serde_json::to_string(&capabilities).unwrap();
    let deserialized: TransportCapabilities = serde_json::from_str(&serialized).unwrap();

    assert_eq!(capabilities.max_message_size, deserialized.max_message_size);
    assert_eq!(
        capabilities.supports_compression,
        deserialized.supports_compression
    );
}

#[test]
fn test_transport_capabilities_debug() {
    let capabilities = TransportCapabilities::default();
    let debug_str = format!("{capabilities:?}");
    assert!(debug_str.contains("TransportCapabilities"));
}

#[test]
fn test_transport_capabilities_clone() {
    let original = TransportCapabilities {
        max_message_size: Some(4096),
        supports_streaming: true,
        compression_algorithms: vec!["deflate".to_string()],
        ..Default::default()
    };

    let cloned = original.clone();
    assert_eq!(original.max_message_size, cloned.max_message_size);
    assert_eq!(original.supports_streaming, cloned.supports_streaming);
    assert_eq!(
        original.compression_algorithms,
        cloned.compression_algorithms
    );
}

#[test]
fn test_transport_capabilities_equality() {
    let caps1 = TransportCapabilities {
        max_message_size: Some(1024),
        supports_compression: true,
        ..Default::default()
    };

    let caps2 = TransportCapabilities {
        max_message_size: Some(1024),
        supports_compression: true,
        ..Default::default()
    };

    let caps3 = TransportCapabilities {
        max_message_size: Some(2048),
        supports_compression: true,
        ..Default::default()
    };

    assert_eq!(caps1, caps2);
    assert_ne!(caps1, caps3);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_transport_types_comprehensive() {
    let all_types = vec![
        TransportType::Stdio,
        TransportType::Http,
        TransportType::WebSocket,
        TransportType::Tcp,
        TransportType::Unix,
    ];

    for transport_type in all_types {
        // Test each type can be serialized and deserialized
        let serialized = serde_json::to_string(&transport_type).unwrap();
        let deserialized: TransportType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(transport_type, deserialized);

        // Test debug output
        let debug_str = format!("{transport_type:?}");
        assert!(!debug_str.is_empty());

        // Test hash (by using in HashSet)
        let mut set = std::collections::HashSet::new();
        set.insert(transport_type);
        assert!(set.contains(&transport_type));
    }
}

#[test]
fn test_transport_states_comprehensive() {
    let all_states = vec![
        TransportState::Disconnected,
        TransportState::Connecting,
        TransportState::Connected,
        TransportState::Disconnecting,
        TransportState::Failed {
            reason: "test failure".to_string(),
        },
    ];

    for state in all_states {
        // Test serialization
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: TransportState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized);

        // Test debug output
        let debug_str = format!("{state:?}");
        assert!(!debug_str.is_empty());

        // Test clone
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }
}

#[test]
fn test_transport_errors_comprehensive() {
    let all_errors = vec![
        TransportError::ConnectionFailed("failed".to_string()),
        TransportError::ConnectionLost("lost".to_string()),
        TransportError::SendFailed("send error".to_string()),
        TransportError::ReceiveFailed("receive error".to_string()),
        TransportError::SerializationFailed("serialization error".to_string()),
        TransportError::ProtocolError("protocol error".to_string()),
        TransportError::Timeout,
        TransportError::ConfigurationError("config error".to_string()),
        TransportError::AuthenticationFailed("auth failed".to_string()),
        TransportError::RateLimitExceeded,
        TransportError::NotAvailable("not available".to_string()),
        TransportError::Io("io error".to_string()),
        TransportError::Internal("internal error".to_string()),
    ];

    for error in all_errors {
        // Test display formatting
        let error_str = format!("{error}");
        assert!(!error_str.is_empty());

        // Test debug formatting
        let debug_str = format!("{error:?}");
        assert!(!debug_str.is_empty());

        // Test clone
        let cloned = error.clone();
        let cloned_str = format!("{cloned}");
        assert_eq!(error_str, cloned_str);
    }
}

#[test]
fn test_transport_capabilities_feature_matrix() {
    // Test different capability combinations that might be realistic
    let configs = vec![
        // Basic stdio-like
        TransportCapabilities {
            supports_streaming: true,
            supports_bidirectional: true,
            ..Default::default()
        },
        // HTTP-like
        TransportCapabilities {
            max_message_size: Some(10 * 1024 * 1024), // 10MB
            supports_compression: true,
            compression_algorithms: vec!["gzip".to_string(), "deflate".to_string()],
            ..Default::default()
        },
        // WebSocket-like
        TransportCapabilities {
            supports_streaming: true,
            supports_bidirectional: true,
            supports_multiplexing: false,
            ..Default::default()
        },
        // Advanced transport
        TransportCapabilities {
            max_message_size: Some(1024 * 1024), // 1MB
            supports_compression: true,
            supports_streaming: true,
            supports_bidirectional: true,
            supports_multiplexing: true,
            compression_algorithms: vec!["gzip".to_string(), "brotli".to_string()],
            custom: {
                let mut custom = HashMap::new();
                custom.insert("encryption".to_string(), serde_json::json!("tls"));
                custom.insert("version".to_string(), serde_json::json!("1.2"));
                custom
            },
        },
    ];

    for capabilities in configs {
        // Test serialization roundtrip
        let serialized = serde_json::to_string(&capabilities).unwrap();
        let deserialized: TransportCapabilities = serde_json::from_str(&serialized).unwrap();
        assert_eq!(capabilities, deserialized);

        // Test that capabilities are internally consistent
        if capabilities.supports_compression {
            // If compression is supported, algorithms might be available
            // (not required, but common)
        }

        if capabilities.supports_multiplexing {
            // Multiplexing usually implies bidirectional
            // (though not always required)
        }
    }
}

// ============================================================================
// Error Message Tests
// ============================================================================

#[test]
fn test_transport_error_messages_specific() {
    let test_cases = vec![
        (
            TransportError::ConnectionFailed("network unreachable".to_string()),
            "Connection failed: network unreachable",
        ),
        (TransportError::Timeout, "Operation timed out"),
        (TransportError::RateLimitExceeded, "Rate limit exceeded"),
        (
            TransportError::AuthenticationFailed("invalid token".to_string()),
            "Authentication failed: invalid token",
        ),
        (
            TransportError::ProtocolError("invalid json".to_string()),
            "Protocol error: invalid json",
        ),
    ];

    for (error, expected_message) in test_cases {
        let actual_message = format!("{error}");
        assert_eq!(actual_message, expected_message);
    }
}

#[test]
fn test_transport_state_transitions() {
    // Test logical state transitions make sense
    let transitions = vec![
        (TransportState::Disconnected, TransportState::Connecting),
        (TransportState::Connecting, TransportState::Connected),
        (TransportState::Connected, TransportState::Disconnecting),
        (TransportState::Disconnecting, TransportState::Disconnected),
        (
            TransportState::Connected,
            TransportState::Failed {
                reason: "error".to_string(),
            },
        ),
        (
            TransportState::Connecting,
            TransportState::Failed {
                reason: "timeout".to_string(),
            },
        ),
    ];

    for (from_state, to_state) in transitions {
        // Verify both states can be created and are different (except for same variants)
        assert_ne!(
            std::mem::discriminant(&from_state),
            std::mem::discriminant(&to_state)
        );

        // Verify both can be serialized
        let _from_json = serde_json::to_string(&from_state).unwrap();
        let _to_json = serde_json::to_string(&to_state).unwrap();
    }
}

#[test]
fn test_transport_type_string_representations() {
    let type_strings = vec![
        (TransportType::Stdio, "stdio"),
        (TransportType::Http, "http"),
        (TransportType::WebSocket, "websocket"),
        (TransportType::Tcp, "tcp"),
        (TransportType::Unix, "unix"),
    ];

    for (transport_type, expected_string) in type_strings {
        let serialized = serde_json::to_string(&transport_type).unwrap();
        assert_eq!(serialized, format!("\"{expected_string}\""));

        let deserialized: TransportType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, transport_type);
    }
}
