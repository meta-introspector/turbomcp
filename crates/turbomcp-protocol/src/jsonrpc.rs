//! # JSON-RPC 2.0 Implementation
//!
//! This module provides a complete implementation of JSON-RPC 2.0 protocol
//! with support for batching, streaming, and MCP-specific extensions.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;

use crate::types::RequestId;

/// JSON-RPC version constant
pub const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC version type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcVersion;

impl Serialize for JsonRpcVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(JSONRPC_VERSION)
    }
}

impl<'de> Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version = String::deserialize(deserializer)?;
        if version == JSONRPC_VERSION {
            Ok(JsonRpcVersion)
        } else {
            Err(serde::de::Error::custom(format!(
                "Invalid JSON-RPC version: expected '{JSONRPC_VERSION}', got '{version}'"
            )))
        }
    }
}

/// JSON-RPC request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version
    pub jsonrpc: JsonRpcVersion,
    /// Request method name
    pub method: String,
    /// Request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Request identifier
    pub id: RequestId,
}

/// JSON-RPC response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: JsonRpcVersion,
    /// Response result (success case)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Response error (error case)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request identifier (null for parse errors)
    pub id: Option<RequestId>,
}

/// JSON-RPC notification message (no response expected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version
    pub jsonrpc: JsonRpcVersion,
    /// Notification method name
    pub method: String,
    /// Notification parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC batch request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsonRpcBatch<T> {
    /// Batch items
    pub items: Vec<T>,
}

/// Standard JSON-RPC error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonRpcErrorCode {
    /// Parse error (-32700)
    ParseError,
    /// Invalid request (-32600)
    InvalidRequest,
    /// Method not found (-32601)
    MethodNotFound,
    /// Invalid params (-32602)
    InvalidParams,
    /// Internal error (-32603)
    InternalError,
    /// Application-defined error
    ApplicationError(i32),
}

impl JsonRpcErrorCode {
    /// Get the numeric error code
    pub fn code(&self) -> i32 {
        match self {
            Self::ParseError => -32700,
            Self::InvalidRequest => -32600,
            Self::MethodNotFound => -32601,
            Self::InvalidParams => -32602,
            Self::InternalError => -32603,
            Self::ApplicationError(code) => *code,
        }
    }

    /// Get the standard error message
    pub fn message(&self) -> &'static str {
        match self {
            Self::ParseError => "Parse error",
            Self::InvalidRequest => "Invalid Request",
            Self::MethodNotFound => "Method not found",
            Self::InvalidParams => "Invalid params",
            Self::InternalError => "Internal error",
            Self::ApplicationError(_) => "Application error",
        }
    }
}

impl fmt::Display for JsonRpcErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message(), self.code())
    }
}

impl From<JsonRpcErrorCode> for JsonRpcError {
    fn from(code: JsonRpcErrorCode) -> Self {
        Self {
            code: code.code(),
            message: code.message().to_string(),
            data: None,
        }
    }
}

impl From<i32> for JsonRpcErrorCode {
    fn from(code: i32) -> Self {
        match code {
            -32700 => Self::ParseError,
            -32600 => Self::InvalidRequest,
            -32601 => Self::MethodNotFound,
            -32602 => Self::InvalidParams,
            -32603 => Self::InternalError,
            other => Self::ApplicationError(other),
        }
    }
}

/// JSON-RPC message type (union of request, response, notification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// Request message
    Request(JsonRpcRequest),
    /// Response message
    Response(JsonRpcResponse),
    /// Notification message
    Notification(JsonRpcNotification),
    /// Batch of messages
    RequestBatch(JsonRpcBatch<JsonRpcRequest>),
    /// Batch of responses
    ResponseBatch(JsonRpcBatch<JsonRpcResponse>),
    /// Mixed batch
    MessageBatch(JsonRpcBatch<JsonRpcMessage>),
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: String, params: Option<Value>, id: RequestId) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            method,
            params,
            id,
        }
    }

    /// Create a request with no parameters
    pub fn without_params(method: String, id: RequestId) -> Self {
        Self::new(method, None, id)
    }

    /// Create a request with parameters
    pub fn with_params<P: Serialize>(
        method: String,
        params: P,
        id: RequestId,
    ) -> Result<Self, serde_json::Error> {
        let params_value = serde_json::to_value(params)?;
        Ok(Self::new(method, Some(params_value), id))
    }
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(result: Value, id: RequestId) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    /// Create an error response
    pub fn error(error: JsonRpcError, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            result: None,
            error: Some(error),
            id,
        }
    }

    /// Create a parse error response (id is null)
    pub fn parse_error(message: Option<String>) -> Self {
        let error = JsonRpcError {
            code: JsonRpcErrorCode::ParseError.code(),
            message: message.unwrap_or_else(|| JsonRpcErrorCode::ParseError.message().to_string()),
            data: None,
        };
        Self::error(error, None)
    }

    /// Check if this is a successful response
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification
    pub fn new(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            method,
            params,
        }
    }

    /// Create a notification with no parameters
    pub fn without_params(method: String) -> Self {
        Self::new(method, None)
    }

    /// Create a notification with parameters
    pub fn with_params<P: Serialize>(method: String, params: P) -> Result<Self, serde_json::Error> {
        let params_value = serde_json::to_value(params)?;
        Ok(Self::new(method, Some(params_value)))
    }
}

impl<T> JsonRpcBatch<T> {
    /// Create a new batch
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    /// Create an empty batch
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Add an item to the batch
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Get the number of items in the batch
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate over batch items
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}

impl<T> IntoIterator for JsonRpcBatch<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T> From<Vec<T>> for JsonRpcBatch<T> {
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

/// Utility functions for JSON-RPC message handling
pub mod utils {
    use super::*;

    /// Parse a JSON-RPC message from a string
    pub fn parse_message(json: &str) -> Result<JsonRpcMessage, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize a JSON-RPC message to a string
    pub fn serialize_message(message: &JsonRpcMessage) -> Result<String, serde_json::Error> {
        serde_json::to_string(message)
    }

    /// Check if a string looks like a JSON-RPC batch
    pub fn is_batch(json: &str) -> bool {
        json.trim_start().starts_with('[')
    }

    /// Extract the method name from a JSON-RPC message string
    pub fn extract_method(json: &str) -> Option<String> {
        // Simple regex-free method extraction for performance
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json)
            && let Some(method) = value.get("method")
        {
            return method.as_str().map(String::from);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_version() {
        let version = JsonRpcVersion;
        let json = serde_json::to_string(&version).unwrap();
        assert_eq!(json, "\"2.0\"");

        let parsed: JsonRpcVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, version);
    }

    #[test]
    fn test_request_creation() {
        let request = JsonRpcRequest::new(
            "test_method".to_string(),
            Some(json!({"key": "value"})),
            RequestId::String("test-id".to_string()),
        );

        assert_eq!(request.method, "test_method");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_response_creation() {
        let response = JsonRpcResponse::success(
            json!({"result": "success"}),
            RequestId::String("test-id".to_string()),
        );

        assert!(response.is_success());
        assert!(!response.is_error());
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_error_response() {
        let error = JsonRpcError::from(JsonRpcErrorCode::MethodNotFound);
        let response =
            JsonRpcResponse::error(error, Some(RequestId::String("test-id".to_string())));

        assert!(!response.is_success());
        assert!(response.is_error());
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_notification() {
        let notification = JsonRpcNotification::without_params("test_notification".to_string());
        assert_eq!(notification.method, "test_notification");
        assert!(notification.params.is_none());
    }

    #[test]
    fn test_batch() {
        let mut batch = JsonRpcBatch::<JsonRpcRequest>::empty();
        assert!(batch.is_empty());

        batch.push(JsonRpcRequest::without_params(
            "method1".to_string(),
            RequestId::String("1".to_string()),
        ));
        batch.push(JsonRpcRequest::without_params(
            "method2".to_string(),
            RequestId::String("2".to_string()),
        ));

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_serialization() {
        let request = JsonRpcRequest::new(
            "test_method".to_string(),
            Some(json!({"param": "value"})),
            RequestId::String("123".to_string()),
        );

        let json = serde_json::to_string(&request).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.method, request.method);
        assert_eq!(parsed.params, request.params);
    }

    #[test]
    fn test_utils() {
        let json = r#"{"jsonrpc":"2.0","method":"test","id":"123"}"#;

        assert!(!utils::is_batch(json));
        assert_eq!(utils::extract_method(json), Some("test".to_string()));

        let batch_json = r#"[{"jsonrpc":"2.0","method":"test","id":"123"}]"#;
        assert!(utils::is_batch(batch_json));
    }

    #[test]
    fn test_error_codes() {
        let parse_error = JsonRpcErrorCode::ParseError;
        assert_eq!(parse_error.code(), -32700);
        assert_eq!(parse_error.message(), "Parse error");

        let app_error = JsonRpcErrorCode::ApplicationError(-32001);
        assert_eq!(app_error.code(), -32001);
    }
}
