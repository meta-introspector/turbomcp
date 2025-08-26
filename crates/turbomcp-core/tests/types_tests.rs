//! Comprehensive tests for core protocol types and data structures

use chrono::{TimeZone, Timelike, Utc};
use std::collections::HashMap;
use std::collections::HashSet;
use turbomcp_core::types::{ContentType, ProtocolVersion, Timestamp};

// ProtocolVersion tests
#[test]
fn test_protocol_version_new() {
    let version = ProtocolVersion::new("1.2.3");
    assert_eq!(version.as_str(), "1.2.3");
}

#[test]
fn test_protocol_version_new_string() {
    let version_str = String::from("2.0.0");
    let version = ProtocolVersion::new(version_str);
    assert_eq!(version.as_str(), "2.0.0");
}

#[test]
fn test_protocol_version_as_str() {
    let version = ProtocolVersion::new("1.0.0");
    assert_eq!(version.as_str(), "1.0.0");
}

#[test]
fn test_protocol_version_default() {
    let version = ProtocolVersion::default();
    // Should use the crate's PROTOCOL_VERSION constant
    assert!(!version.as_str().is_empty());
}

#[test]
fn test_protocol_version_display() {
    let version = ProtocolVersion::new("1.5.0");
    let displayed = format!("{version}");
    assert_eq!(displayed, "1.5.0");
}

#[test]
fn test_protocol_version_debug() {
    let version = ProtocolVersion::new("1.0.0");
    let debug_str = format!("{version:?}");
    assert!(debug_str.contains("ProtocolVersion"));
    assert!(debug_str.contains("1.0.0"));
}

#[test]
fn test_protocol_version_clone() {
    let original = ProtocolVersion::new("1.0.0");
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_protocol_version_equality() {
    let version1 = ProtocolVersion::new("1.0.0");
    let version2 = ProtocolVersion::new("1.0.0");
    let version3 = ProtocolVersion::new("2.0.0");

    assert_eq!(version1, version2);
    assert_ne!(version1, version3);
}

#[test]
fn test_protocol_version_hash() {
    let version1 = ProtocolVersion::new("1.0.0");
    let version2 = ProtocolVersion::new("1.0.0");
    let version3 = ProtocolVersion::new("2.0.0");

    let mut set = HashSet::new();
    set.insert(version1);
    set.insert(version2); // Should not be added since it's equal to version1
    set.insert(version3);

    assert_eq!(set.len(), 2);
}

#[test]
fn test_protocol_version_from_str() {
    let version = ProtocolVersion::from("3.0.0");
    assert_eq!(version.as_str(), "3.0.0");
}

#[test]
fn test_protocol_version_from_string() {
    let version = ProtocolVersion::from(String::from("4.0.0"));
    assert_eq!(version.as_str(), "4.0.0");
}

#[test]
fn test_protocol_version_serialization() {
    let version = ProtocolVersion::new("1.2.3");
    let json = serde_json::to_string(&version).unwrap();
    assert!(json.contains("1.2.3"));

    let deserialized: ProtocolVersion = serde_json::from_str(&json).unwrap();
    assert_eq!(version, deserialized);
}

#[test]
fn test_protocol_version_empty_string() {
    let version = ProtocolVersion::new("");
    assert_eq!(version.as_str(), "");
}

#[test]
fn test_protocol_version_special_characters() {
    let version = ProtocolVersion::new("1.0.0-alpha+build.123");
    assert_eq!(version.as_str(), "1.0.0-alpha+build.123");
}

// Timestamp tests
#[test]
fn test_timestamp_now() {
    let timestamp = Timestamp::now();
    let elapsed = timestamp.elapsed();

    // Should be very recent (within 1 second)
    assert!(elapsed.num_seconds() < 1);
}

#[test]
fn test_timestamp_from_datetime() {
    let dt = Utc.with_ymd_and_hms(2023, 12, 25, 12, 0, 0).unwrap();
    let timestamp = Timestamp::from_datetime(dt);
    assert_eq!(timestamp.datetime(), dt);
}

#[test]
fn test_timestamp_datetime() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let timestamp = Timestamp::from_datetime(dt);
    assert_eq!(timestamp.datetime(), dt);
}

#[test]
fn test_timestamp_elapsed() {
    let past_dt = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    let timestamp = Timestamp::from_datetime(past_dt);
    let elapsed = timestamp.elapsed();

    // Should be positive (in the past)
    assert!(elapsed.num_seconds() > 0);
}

#[test]
fn test_timestamp_display() {
    let dt = Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 45).unwrap();
    let timestamp = Timestamp::from_datetime(dt);
    let displayed = format!("{timestamp}");

    // Should be in RFC3339 format
    assert!(displayed.contains("2024-06-15"));
    assert!(displayed.contains("14:30:45"));
}

#[test]
fn test_timestamp_debug() {
    let timestamp = Timestamp::now();
    let debug_str = format!("{timestamp:?}");
    assert!(debug_str.contains("Timestamp"));
}

#[test]
fn test_timestamp_clone() {
    let original = Timestamp::now();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_timestamp_equality() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let timestamp1 = Timestamp::from_datetime(dt);
    let timestamp2 = Timestamp::from_datetime(dt);
    let timestamp3 = Timestamp::now();

    assert_eq!(timestamp1, timestamp2);
    assert_ne!(timestamp1, timestamp3);
}

#[test]
fn test_timestamp_ordering() {
    let dt1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let dt2 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();

    let timestamp1 = Timestamp::from_datetime(dt1);
    let timestamp2 = Timestamp::from_datetime(dt2);

    assert!(timestamp1 < timestamp2);
    assert!(timestamp2 > timestamp1);
}

#[test]
fn test_timestamp_from_datetime_trait() {
    let dt = Utc.with_ymd_and_hms(2024, 5, 10, 8, 15, 30).unwrap();
    let timestamp: Timestamp = dt.into();
    assert_eq!(timestamp.datetime(), dt);
}

#[test]
fn test_timestamp_serialization() {
    let dt = Utc.with_ymd_and_hms(2024, 3, 20, 10, 45, 0).unwrap();
    let timestamp = Timestamp::from_datetime(dt);

    let json = serde_json::to_string(&timestamp).unwrap();
    let deserialized: Timestamp = serde_json::from_str(&json).unwrap();

    assert_eq!(timestamp, deserialized);
}

// ContentType tests
#[test]
fn test_content_type_variants() {
    let json = ContentType::Json;
    let binary = ContentType::Binary;
    let text = ContentType::Text;

    assert_ne!(json, binary);
    assert_ne!(json, text);
    assert_ne!(binary, text);
}

#[test]
fn test_content_type_debug() {
    let json = ContentType::Json;
    let debug_str = format!("{json:?}");
    assert!(debug_str.contains("Json"));
}

#[test]
fn test_content_type_clone() {
    let original = ContentType::Json;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn test_content_type_copy() {
    let original = ContentType::Binary;
    let copied = original;
    assert_eq!(original, copied);
    assert_eq!(copied, ContentType::Binary);
}

#[test]
fn test_content_type_equality() {
    let json1 = ContentType::Json;
    let json2 = ContentType::Json;
    let binary = ContentType::Binary;

    assert_eq!(json1, json2);
    assert_ne!(json1, binary);
}

#[test]
fn test_content_type_hash() {
    let json = ContentType::Json;
    let binary = ContentType::Binary;
    let text = ContentType::Text;

    let mut set = HashSet::new();
    set.insert(json);
    set.insert(binary);
    set.insert(text);
    set.insert(json); // Duplicate, should not increase size

    assert_eq!(set.len(), 3);
}

#[test]
fn test_content_type_serialization() {
    let json = ContentType::Json;
    let binary = ContentType::Binary;
    let text = ContentType::Text;

    // Test serialization uses lowercase
    let json_str = serde_json::to_string(&json).unwrap();
    let binary_str = serde_json::to_string(&binary).unwrap();
    let text_str = serde_json::to_string(&text).unwrap();

    assert_eq!(json_str, "\"json\"");
    assert_eq!(binary_str, "\"binary\"");
    assert_eq!(text_str, "\"text\"");

    // Test deserialization
    let json_deser: ContentType = serde_json::from_str("\"json\"").unwrap();
    let binary_deser: ContentType = serde_json::from_str("\"binary\"").unwrap();
    let text_deser: ContentType = serde_json::from_str("\"text\"").unwrap();

    assert_eq!(json, json_deser);
    assert_eq!(binary, binary_deser);
    assert_eq!(text, text_deser);
}

#[test]
fn test_content_type_invalid_serialization() {
    let result: Result<ContentType, _> = serde_json::from_str("\"invalid\"");
    assert!(result.is_err());
}

// Integration tests combining multiple types
#[test]
fn test_types_in_hashmap() {
    let mut map = HashMap::new();

    map.insert(ProtocolVersion::new("1.0.0"), ContentType::Json);
    map.insert(ProtocolVersion::new("2.0.0"), ContentType::Binary);

    assert_eq!(map.len(), 2);
    assert_eq!(
        map.get(&ProtocolVersion::new("1.0.0")),
        Some(&ContentType::Json)
    );
}

#[test]
fn test_complex_serialization() {
    use serde_json::json;

    let data = json!({
        "version": ProtocolVersion::new("1.0.0"),
        "timestamp": Timestamp::now(),
        "content_type": ContentType::Json
    });

    let json_str = serde_json::to_string(&data).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(parsed.is_object());
    assert!(parsed["version"].is_string());
    assert!(parsed["timestamp"].is_string());
    assert!(parsed["content_type"].is_string());
}

#[test]
fn test_timestamp_with_microseconds() {
    let dt = Utc
        .with_ymd_and_hms(2024, 1, 1, 12, 0, 0)
        .unwrap()
        .with_nanosecond(123_456_789)
        .unwrap();
    let timestamp = Timestamp::from_datetime(dt);

    assert_eq!(timestamp.datetime(), dt);

    // Test that display includes microseconds
    let displayed = format!("{timestamp}");
    assert!(displayed.contains("2024-01-01T12:00:00"));
}

#[test]
fn test_protocol_version_edge_cases() {
    let versions = vec![
        "0.0.0",
        "999.999.999",
        "1.0.0-beta",
        "2.0.0+build.1",
        "v1.0.0",
        "1.0",
        "1",
    ];

    for version_str in versions {
        let version = ProtocolVersion::new(version_str);
        assert_eq!(version.as_str(), version_str);

        // Test roundtrip serialization
        let json = serde_json::to_string(&version).unwrap();
        let deserialized: ProtocolVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(version, deserialized);
    }
}

#[test]
fn test_timestamp_future() {
    // Create a timestamp in the future
    let future_dt = Utc::now() + chrono::Duration::hours(1);
    let timestamp = Timestamp::from_datetime(future_dt);

    let elapsed = timestamp.elapsed();
    // Should be negative (in the future)
    assert!(elapsed.num_seconds() < 0);
}

#[test]
fn test_all_types_debug_format() {
    let version = ProtocolVersion::new("1.0.0");
    let timestamp = Timestamp::now();
    let content_type = ContentType::Json;

    let debug_version = format!("{version:?}");
    let debug_timestamp = format!("{timestamp:?}");
    let debug_content = format!("{content_type:?}");

    assert!(!debug_version.is_empty());
    assert!(!debug_timestamp.is_empty());
    assert!(!debug_content.is_empty());
}
