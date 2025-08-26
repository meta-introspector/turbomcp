#[cfg(feature = "fancy-errors")]
use serde_json::json;
#[cfg(feature = "fancy-errors")]
use std::collections::HashMap;
#[cfg(feature = "fancy-errors")]
use turbomcp_core::config::{ConfigBuilder, CoreConfig};

#[cfg(feature = "fancy-errors")]
#[test]
fn test_core_config_default() {
    let config = CoreConfig::default();

    assert_eq!(config.max_message_size, 64 * 1024 * 1024); // 64MB
    assert_eq!(config.timeout_ms, 30_000); // 30 seconds
    assert!(config.tracing_enabled);
    assert!(config.options.is_empty());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_core_config_creation() {
    let options = HashMap::new();
    let config = CoreConfig {
        max_message_size: 1024 * 1024, // 1MB
        timeout_ms: 5000,              // 5 seconds
        tracing_enabled: false,
        options,
    };

    assert_eq!(config.max_message_size, 1024 * 1024);
    assert_eq!(config.timeout_ms, 5000);
    assert!(!config.tracing_enabled);
    assert!(config.options.is_empty());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_core_config_clone() {
    let config = CoreConfig::default();
    let cloned = config.clone();

    assert_eq!(config.max_message_size, cloned.max_message_size);
    assert_eq!(config.timeout_ms, cloned.timeout_ms);
    assert_eq!(config.tracing_enabled, cloned.tracing_enabled);
    assert_eq!(config.options.len(), cloned.options.len());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_core_config_debug() {
    let config = CoreConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("CoreConfig"));
    assert!(debug_str.contains("max_message_size"));
    assert!(debug_str.contains("timeout_ms"));
    assert!(debug_str.contains("tracing_enabled"));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_core_config_serialization() {
    let config = CoreConfig::default();

    // Test serialization
    let serialized = serde_json::to_string(&config).expect("Should serialize");
    assert!(serialized.contains("max_message_size"));
    assert!(serialized.contains("timeout_ms"));
    assert!(serialized.contains("tracing_enabled"));

    // Test deserialization
    let deserialized: CoreConfig = serde_json::from_str(&serialized).expect("Should deserialize");
    assert_eq!(config.max_message_size, deserialized.max_message_size);
    assert_eq!(config.timeout_ms, deserialized.timeout_ms);
    assert_eq!(config.tracing_enabled, deserialized.tracing_enabled);
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_new() {
    let builder = ConfigBuilder::new();
    let config = builder.build();

    // Should match default values
    assert_eq!(config.max_message_size, 64 * 1024 * 1024);
    assert_eq!(config.timeout_ms, 30_000);
    assert!(config.tracing_enabled);
    assert!(config.options.is_empty());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_default() {
    let builder = ConfigBuilder::default();
    let config = builder.build();

    // Should match default values
    assert_eq!(config.max_message_size, 64 * 1024 * 1024);
    assert_eq!(config.timeout_ms, 30_000);
    assert!(config.tracing_enabled);
    assert!(config.options.is_empty());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_max_message_size_valid() {
    let config = ConfigBuilder::new()
        .max_message_size(1024 * 1024)
        .expect("Should set valid size")
        .build();

    assert_eq!(config.max_message_size, 1024 * 1024);
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_max_message_size_zero() {
    let result = ConfigBuilder::new().max_message_size(0);

    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("cannot be zero"));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_max_message_size_too_large() {
    let result = ConfigBuilder::new().max_message_size(2 * 1024 * 1024 * 1024); // 2GB

    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("cannot exceed 1GB"));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_max_message_size_boundary() {
    // Test exactly 1GB (should succeed - implementation uses >)
    let result = ConfigBuilder::new().max_message_size(1024 * 1024 * 1024);
    assert!(result.is_ok());

    // Test just over 1GB (should fail)
    let result = ConfigBuilder::new().max_message_size(1024 * 1024 * 1024 + 1);
    assert!(result.is_err());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_timeout_valid() {
    let config = ConfigBuilder::new()
        .timeout_ms(5000)
        .expect("Should set valid timeout")
        .build();

    assert_eq!(config.timeout_ms, 5000);
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_timeout_zero() {
    let result = ConfigBuilder::new().timeout_ms(0);

    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("cannot be zero"));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_timeout_too_large() {
    let result = ConfigBuilder::new().timeout_ms(11 * 60 * 1000); // 11 minutes

    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("cannot exceed 10 minutes"));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_timeout_boundary() {
    // Test exactly 10 minutes (should succeed - implementation uses >)
    let result = ConfigBuilder::new().timeout_ms(10 * 60 * 1000);
    assert!(result.is_ok());

    // Test just over 10 minutes (should fail)
    let result = ConfigBuilder::new().timeout_ms(10 * 60 * 1000 + 1);
    assert!(result.is_err());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_tracing_enabled() {
    let config_enabled = ConfigBuilder::new().tracing_enabled(true).build();

    let config_disabled = ConfigBuilder::new().tracing_enabled(false).build();

    assert!(config_enabled.tracing_enabled);
    assert!(!config_disabled.tracing_enabled);
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_option_string() {
    let config = ConfigBuilder::new()
        .option("test_key", "test_value")
        .expect("Should add string option")
        .build();

    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options.get("test_key"), Some(&json!("test_value")));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_option_number() {
    let config = ConfigBuilder::new()
        .option("number", 42)
        .expect("Should add number option")
        .build();

    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options.get("number"), Some(&json!(42)));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_option_boolean() {
    let config = ConfigBuilder::new()
        .option("flag", true)
        .expect("Should add boolean option")
        .build();

    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options.get("flag"), Some(&json!(true)));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_option_json_object() {
    let json_obj = json!({
        "nested": {
            "key": "value",
            "count": 123
        }
    });

    let config = ConfigBuilder::new()
        .option("complex", &json_obj)
        .expect("Should add JSON object option")
        .build();

    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options.get("complex"), Some(&json_obj));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_multiple_options() {
    let config = ConfigBuilder::new()
        .option("string", "value")
        .expect("Should add string")
        .option("number", 100)
        .expect("Should add number")
        .option("boolean", false)
        .expect("Should add boolean")
        .build();

    assert_eq!(config.options.len(), 3);
    assert_eq!(config.options.get("string"), Some(&json!("value")));
    assert_eq!(config.options.get("number"), Some(&json!(100)));
    assert_eq!(config.options.get("boolean"), Some(&json!(false)));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_chaining() {
    let config = ConfigBuilder::new()
        .max_message_size(1024)
        .expect("Should set size")
        .timeout_ms(5000)
        .expect("Should set timeout")
        .tracing_enabled(false)
        .option("key", "value")
        .expect("Should add option")
        .build();

    assert_eq!(config.max_message_size, 1024);
    assert_eq!(config.timeout_ms, 5000);
    assert!(!config.tracing_enabled);
    assert_eq!(config.options.len(), 1);
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_debug() {
    let builder = ConfigBuilder::new();
    let debug_str = format!("{builder:?}");

    assert!(debug_str.contains("ConfigBuilder"));
    assert!(debug_str.contains("config"));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_builder_error_handling() {
    // Test that errors are properly propagated
    let result = ConfigBuilder::new()
        .max_message_size(1024)
        .expect("Should set size")
        .max_message_size(0); // This should fail

    assert!(result.is_err());
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_options_override() {
    let config = ConfigBuilder::new()
        .option("key", "value1")
        .expect("Should add first value")
        .option("key", "value2") // Override with second value
        .expect("Should add second value")
        .build();

    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options.get("key"), Some(&json!("value2")));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_all_field_types() {
    let mut options = HashMap::new();
    options.insert("test".to_string(), json!("test_value"));

    let config = CoreConfig {
        max_message_size: 2048,
        timeout_ms: 10000,
        tracing_enabled: false,
        options,
    };

    // Test all fields are properly set
    assert_eq!(config.max_message_size, 2048);
    assert_eq!(config.timeout_ms, 10000);
    assert!(!config.tracing_enabled);
    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options.get("test"), Some(&json!("test_value")));
}

#[cfg(feature = "fancy-errors")]
#[test]
fn test_config_realistic_values() {
    // Test with realistic configuration values
    let config = ConfigBuilder::new()
        .max_message_size(16 * 1024 * 1024) // 16MB
        .expect("Should set realistic size")
        .timeout_ms(60 * 1000) // 1 minute
        .expect("Should set realistic timeout")
        .tracing_enabled(true)
        .option("environment", "production")
        .expect("Should add environment")
        .option("max_connections", 1000)
        .expect("Should add max connections")
        .build();

    assert_eq!(config.max_message_size, 16 * 1024 * 1024);
    assert_eq!(config.timeout_ms, 60 * 1000);
    assert!(config.tracing_enabled);
    assert_eq!(config.options.len(), 2);
}
