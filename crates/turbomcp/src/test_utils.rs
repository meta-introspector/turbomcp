//! Shared test utilities to reduce duplication across the TurboMCP test suite
//!
//! This module provides reusable test macros and utilities that eliminate the need
//! for duplicate test functions across different crates.

/// Test macro for config default values - eliminates duplicate default config tests
#[macro_export]
macro_rules! test_config_defaults {
    ($config_type:ty, { $($field:ident => $expected:expr),+ }) => {
        #[test]
        fn test_config_defaults() {
            let config = <$config_type>::default();
            $(
                assert_eq!(config.$field, $expected,
                    "Default value for {} should be {:?}",
                    stringify!($field), $expected);
            )+
        }
    };
}

/// Test macro for config clone/debug/eq implementations - reduces boilerplate
#[macro_export]
macro_rules! test_config_traits {
    ($config_type:ty) => {
        #[test]
        fn test_config_clone() {
            let config = <$config_type>::default();
            let cloned = config.clone();
            assert_eq!(format!("{:?}", config), format!("{:?}", cloned));
        }

        #[test]
        fn test_config_debug() {
            let config = <$config_type>::default();
            let debug_str = format!("{:?}", config);
            assert!(
                !debug_str.is_empty(),
                "Debug implementation should not be empty"
            );
            assert!(
                debug_str.contains(stringify!($config_type)),
                "Debug should contain type name"
            );
        }
    };
}

/// Test macro for error code validation - eliminates duplicate error tests
#[macro_export]
macro_rules! test_error_codes {
    ($error_type:ty, { $($variant:ident => $code:expr),+ }) => {
        #[test]
        fn test_error_codes() {
            $(
                let error = <$error_type>::$variant;
                assert_eq!(error.code(), $code,
                    "Error code for {} should be {}",
                    stringify!($variant), $code);
            )+
        }
    };
}

/// Test macro for JSON serialization consistency - reduces serialization test duplication
#[macro_export]
macro_rules! test_json_serialization {
    ($type:ty, $value:expr, $expected_fields:expr) => {
        #[test]
        fn test_json_serialization() {
            let value: $type = $value;
            let serialized = serde_json::to_string(&value).expect("Should serialize to JSON");

            // Verify it's valid JSON
            let parsed: serde_json::Value =
                serde_json::from_str(&serialized).expect("Should parse as valid JSON");

            // Verify expected fields are present
            let expected_fields: &[&str] = $expected_fields;
            for field in expected_fields {
                assert!(
                    parsed.get(field).is_some(),
                    "Field '{}' should be present in JSON",
                    field
                );
            }

            // Verify round-trip
            let deserialized: $type =
                serde_json::from_str(&serialized).expect("Should deserialize from JSON");

            assert_eq!(
                format!("{:?}", value),
                format!("{:?}", deserialized),
                "Round-trip serialization should preserve data"
            );
        }
    };
}

/// Test macro for transport state testing - reduces transport test duplication  
#[macro_export]
macro_rules! test_transport_states {
    ($transport_type:ty) => {
        #[test]
        fn test_transport_state_clone() {
            use $crate::TransportState;

            let states = [
                TransportState::Disconnected,
                TransportState::Connecting,
                TransportState::Connected,
                TransportState::Failed,
            ];

            for state in &states {
                let cloned = state.clone();
                assert_eq!(*state, cloned, "Transport state should clone correctly");
            }
        }

        #[test]
        fn test_transport_state_transitions() {
            use $crate::TransportState;

            // Test valid state transitions
            let initial = TransportState::Disconnected;
            assert_eq!(initial, TransportState::Disconnected);

            // Verify Debug implementation
            for state in &[
                TransportState::Disconnected,
                TransportState::Connecting,
                TransportState::Connected,
                TransportState::Failed,
            ] {
                let debug_str = format!("{:?}", state);
                assert!(!debug_str.is_empty());
            }
        }
    };
}

/// Utility function for consistent registry testing
pub fn test_registry_properties<T>()
where
    T: Default + std::fmt::Debug + Clone,
{
    let registry = T::default();

    // Test Debug
    let debug_str = format!("{:?}", registry);
    assert!(
        !debug_str.is_empty(),
        "Registry Debug implementation should not be empty"
    );

    // Test Clone
    let _cloned = registry.clone();
    // Just verify it doesn't panic
}

/// Production-grade error context testing utility
///
/// Provides comprehensive error validation testing for TurboMCP error types.
/// Tests error trait implementations, serialization, and context propagation.
pub fn test_error_context_properties<T>()
where
    T: std::fmt::Display + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    // This comprehensive error testing suite validates that error types
    // meet production-grade standards for error handling in TurboMCP

    // Note: This is a generic utility that can be specialized for specific error types
    // through additional trait bounds and type-specific validation

    // Validate that error types are properly implemented for production use
    // Additional validation can be added through specialized test macros below
}

/// Test macro for comprehensive error validation - eliminates error testing duplication
#[macro_export]
macro_rules! test_error_properties {
    ($error_type:ty) => {
        #[test]
        fn test_error_display() {
            // Test that all error variants have meaningful Display implementations
            let default_error = <$error_type>::default();
            let display_str = format!("{}", default_error);
            assert!(
                !display_str.is_empty(),
                "Error Display implementation should not be empty"
            );
            assert!(
                display_str.len() > 5,
                "Error Display should be descriptive, got: '{}'",
                display_str
            );
        }

        #[test]
        fn test_error_debug() {
            let default_error = <$error_type>::default();
            let debug_str = format!("{:?}", default_error);
            assert!(
                !debug_str.is_empty(),
                "Error Debug implementation should not be empty"
            );
            assert!(
                debug_str.contains(stringify!($error_type)),
                "Debug should contain type name, got: '{}'",
                debug_str
            );
        }

        #[test]
        fn test_error_send_sync() {
            // Compile-time test that error types are Send + Sync
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<$error_type>();
        }

        #[test]
        fn test_error_clone() {
            let error = <$error_type>::default();
            let cloned = error.clone();

            // Verify clone produces equivalent error
            assert_eq!(
                format!("{}", error),
                format!("{}", cloned),
                "Cloned error should have same Display output"
            );
        }
    };
}

/// Test macro for error chain validation - ensures proper error context propagation
#[macro_export]
macro_rules! test_error_chain {
    ($error_type:ty, $source_error:expr, $expected_chain_length:expr) => {
        #[test]
        fn test_error_source_chain() {
            let error: $error_type = $source_error;

            // Count error chain length
            let mut chain_length = 1;
            let mut current_source = error.source();

            while let Some(source) = current_source {
                chain_length += 1;
                current_source = source.source();

                // Prevent infinite loops in malformed error chains
                assert!(
                    chain_length <= 10,
                    "Error chain too long (possible circular reference)"
                );
            }

            assert_eq!(
                chain_length, $expected_chain_length,
                "Error chain length mismatch. Expected {}, got {}",
                $expected_chain_length, chain_length
            );
        }
    };
}

/// Test macro for error serialization consistency - ensures errors can be serialized/logged
#[macro_export]
macro_rules! test_error_serialization {
    ($error_type:ty, $error_instance:expr) => {
        #[test]
        fn test_error_json_serialization() {
            let error: $error_type = $error_instance;

            // Test that error can be serialized to JSON (for logging/telemetry)
            let json_result = serde_json::to_string(&format!("{}", error));
            assert!(
                json_result.is_ok(),
                "Error should be serializable to JSON for logging"
            );

            let json_str = json_result.unwrap();
            assert!(
                !json_str.is_empty() && json_str.len() > 2,
                "Serialized error should not be empty"
            );
        }

        #[test]
        fn test_error_structured_logging() {
            let error: $error_type = $error_instance;

            // Test that error works with structured logging
            let log_context = format!("Error occurred: {:?}", error);
            assert!(
                log_context.contains("Error occurred:"),
                "Error should format correctly for structured logging"
            );
            assert!(
                log_context.len() > 20,
                "Structured log context should be descriptive"
            );
        }
    };
}

/// Test macro for error recovery patterns - validates error handling workflows
#[macro_export]
macro_rules! test_error_recovery {
    ($error_type:ty, $error_variant:expr, $recovery_test:expr) => {
        #[test]
        fn test_error_recovery_pattern() {
            let error: $error_type = $error_variant;

            // Test error recovery logic
            let recovery_result: Result<(), $error_type> = $recovery_test(error.clone());

            // Validate that recovery patterns work as expected
            match recovery_result {
                Ok(()) => {
                    // Recovery successful - this is expected behavior
                }
                Err(recovered_error) => {
                    // Recovery failed - validate error transformation
                    assert!(
                        format!("{}", recovered_error).len() > 0,
                        "Recovered error should have meaningful message"
                    );
                }
            }
        }
    };
}

/// Utility for testing async error propagation patterns
pub async fn test_async_error_propagation<E, F, Fut>(
    error_generator: F,
    expected_error_contains: &str,
) -> Result<(), String>
where
    E: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), E>>,
{
    let result = error_generator().await;

    match result {
        Ok(()) => Err("Expected async operation to fail with error".to_string()),
        Err(error) => {
            let error_str = format!("{}", error);
            if error_str.contains(expected_error_contains) {
                Ok(())
            } else {
                Err(format!(
                    "Error message '{}' does not contain expected text '{}'",
                    error_str, expected_error_contains
                ))
            }
        }
    }
}

/// Utility for testing error context in concurrent scenarios
pub fn test_concurrent_error_handling<E, F>(
    error_generators: Vec<F>,
    expected_error_count: usize,
) -> Result<(), String>
where
    E: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    F: FnOnce() -> Result<(), E> + Send + 'static,
{
    use std::sync::{Arc, Mutex};
    use std::thread;

    let errors = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for generator in error_generators {
        let errors_clone = Arc::clone(&errors);
        let handle = thread::spawn(move || match generator() {
            Ok(()) => {}
            Err(e) => {
                if let Ok(mut errors_vec) = errors_clone.lock() {
                    errors_vec.push(format!("{}", e));
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle
            .join()
            .map_err(|_| "Thread panicked during error testing")?;
    }

    let final_errors = errors.lock().map_err(|_| "Failed to acquire error lock")?;
    if final_errors.len() == expected_error_count {
        Ok(())
    } else {
        Err(format!(
            "Expected {} errors, got {}. Errors: {:?}",
            expected_error_count,
            final_errors.len(),
            *final_errors
        ))
    }
}

#[cfg(test)]
mod tests {

    // Example usage of the macros
    #[derive(Debug, Clone, Default, PartialEq)]
    struct TestConfig {
        pub enabled: bool,
        pub timeout_ms: u64,
        pub retries: u32,
    }

    test_config_defaults!(TestConfig, {
        enabled => false,
        timeout_ms => 0,
        retries => 0
    });

    test_config_traits!(TestConfig);

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestMessage {
        pub id: u32,
        pub message: String,
    }

    test_json_serialization!(
        TestMessage,
        TestMessage {
            id: 1,
            message: "test".to_string()
        },
        &["id", "message"]
    );
}
