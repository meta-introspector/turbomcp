//! Core protocol types and data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Protocol version identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion(pub String);

/// Timestamp wrapper for consistent time handling
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

/// Content type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// JSON content
    Json,
    /// Binary content
    Binary,
    /// Plain text content
    Text,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    /// Get the version string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self(crate::PROTOCOL_VERSION.to_string())
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Timestamp {
    /// Create a new timestamp with current time
    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Create a timestamp from a `DateTime`
    #[must_use]
    pub const fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Get the inner `DateTime`
    #[must_use]
    pub const fn datetime(&self) -> DateTime<Utc> {
        self.0
    }

    /// Get duration since this timestamp
    #[must_use]
    pub fn elapsed(&self) -> chrono::Duration {
        Utc::now() - self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<&str> for ProtocolVersion {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ProtocolVersion {
    fn from(s: String) -> Self {
        Self(s)
    }
}
