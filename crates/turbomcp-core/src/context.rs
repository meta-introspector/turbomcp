//! Request and response context for rich metadata handling.
//!
//! Enhanced with client identification, session management, and request analytics
//! for comprehensive MCP application monitoring and management.
//!
//! # Examples
//!
//! ## Creating a basic request context
//!
//! ```
//! use turbomcp_core::RequestContext;
//!
//! let ctx = RequestContext::new();
//! println!("Request ID: {}", ctx.request_id);
//! assert!(!ctx.request_id.is_empty());
//! ```
//!
//! ## Building a context with metadata
//!
//! ```
//! use turbomcp_core::RequestContext;
//!
//! let ctx = RequestContext::new()
//!     .with_user_id("user123")
//!     .with_session_id("session456")
//!     .with_metadata("api_version", "2.0")
//!     .with_metadata("client", "web_app");
//!
//! assert_eq!(ctx.user_id, Some("user123".to_string()));
//! assert_eq!(ctx.session_id, Some("session456".to_string()));
//! assert_eq!(ctx.get_metadata("api_version"), Some(&serde_json::json!("2.0")));
//! ```
//!
//! ## Working with response contexts
//!
//! ```
//! use turbomcp_core::{RequestContext, ResponseContext};
//! use std::time::Duration;
//!
//! let request_ctx = RequestContext::with_id("req-123");
//! let duration = Duration::from_millis(250);
//!
//! // Successful response
//! let success_ctx = ResponseContext::success(&request_ctx.request_id, duration);
//!
//! // Error response
//! let error_ctx = ResponseContext::error(&request_ctx.request_id, duration, -32600, "Invalid Request");
//!
//! assert_eq!(success_ctx.request_id, "req-123");
//! assert_eq!(error_ctx.request_id, "req-123");
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::types::Timestamp;

/// Context information for request processing
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request identifier
    pub request_id: String,

    /// User identifier (if authenticated)
    pub user_id: Option<String>,

    /// Session identifier
    pub session_id: Option<String>,

    /// Client identifier
    pub client_id: Option<String>,

    /// Request timestamp
    pub timestamp: Timestamp,

    /// Request start time for performance tracking
    pub start_time: Instant,

    /// Custom metadata
    pub metadata: Arc<HashMap<String, serde_json::Value>>,

    /// Tracing span context
    #[cfg(feature = "tracing")]
    pub span: Option<tracing::Span>,

    /// Cancellation token
    pub cancellation_token: Option<Arc<CancellationToken>>,
}

/// Context information for response processing
#[derive(Debug, Clone)]
pub struct ResponseContext {
    /// Original request ID
    pub request_id: String,

    /// Response timestamp
    pub timestamp: Timestamp,

    /// Processing duration
    pub duration: std::time::Duration,

    /// Response status
    pub status: ResponseStatus,

    /// Custom metadata
    pub metadata: Arc<HashMap<String, serde_json::Value>>,
}

/// Response status information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Successful response
    Success,
    /// Error response
    Error {
        /// Error code
        code: i32,
        /// Error message
        message: String,
    },
    /// Partial response (streaming)
    Partial,
    /// Cancelled response
    Cancelled,
}

impl RequestContext {
    /// Create a new request context
    ///
    /// # Examples
    ///
    /// ```
    /// use turbomcp_core::RequestContext;
    ///
    /// let ctx = RequestContext::new();
    /// assert!(!ctx.request_id.is_empty());
    /// assert!(ctx.user_id.is_none());
    /// assert!(ctx.session_id.is_none());
    /// assert!(ctx.metadata.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_id: None,
            session_id: None,
            client_id: None,
            timestamp: Timestamp::now(),
            start_time: Instant::now(),
            metadata: Arc::new(HashMap::new()),
            #[cfg(feature = "tracing")]
            span: None,
            cancellation_token: None,
        }
    }
    /// Return true if the request is authenticated according to context metadata
    ///
    /// # Examples
    ///
    /// ```
    /// use turbomcp_core::RequestContext;
    ///
    /// let ctx = RequestContext::new()
    ///     .with_metadata("authenticated", true);
    /// assert!(ctx.is_authenticated());
    ///
    /// let unauth_ctx = RequestContext::new();
    /// assert!(!unauth_ctx.is_authenticated());
    /// ```
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.metadata
            .get("authenticated")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    }

    /// Return user id if present
    #[must_use]
    pub fn user(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// Return roles from `auth.roles` metadata, if present
    #[must_use]
    pub fn roles(&self) -> Vec<String> {
        self.metadata
            .get("auth")
            .and_then(|v| v.get("roles"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Return true if the user has any of the required roles
    pub fn has_any_role<S: AsRef<str>>(&self, required: &[S]) -> bool {
        if required.is_empty() {
            return true;
        }
        let user_roles = self.roles();
        if user_roles.is_empty() {
            return false;
        }
        let set: std::collections::HashSet<_> = user_roles.into_iter().collect();
        required.iter().any(|r| set.contains(r.as_ref()))
    }

    /// Create a request context with specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            request_id: id.into(),
            ..Self::new()
        }
    }

    /// Set the user ID
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the session ID
    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set the client ID
    #[must_use]
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        Arc::make_mut(&mut self.metadata).insert(key.into(), value.into());
        self
    }

    /// Set cancellation token
    #[must_use]
    pub fn with_cancellation_token(mut self, token: Arc<CancellationToken>) -> Self {
        self.cancellation_token = Some(token);
        self
    }

    /// Get elapsed time since request started
    #[must_use]
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Check if request is cancelled
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token
            .as_ref()
            .is_some_and(|token| token.is_cancelled())
    }

    /// Get metadata value
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Clone with new request ID (for sub-requests)
    #[must_use]
    pub fn derive(&self) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_id: self.user_id.clone(),
            session_id: self.session_id.clone(),
            client_id: self.client_id.clone(),
            timestamp: Timestamp::now(),
            start_time: Instant::now(),
            metadata: self.metadata.clone(),
            #[cfg(feature = "tracing")]
            span: None,
            cancellation_token: self.cancellation_token.clone(),
        }
    }
}

impl ResponseContext {
    /// Create a successful response context
    pub fn success(request_id: impl Into<String>, duration: std::time::Duration) -> Self {
        Self {
            request_id: request_id.into(),
            timestamp: Timestamp::now(),
            duration,
            status: ResponseStatus::Success,
            metadata: Arc::new(HashMap::new()),
        }
    }

    /// Create an error response context
    pub fn error(
        request_id: impl Into<String>,
        duration: std::time::Duration,
        code: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            timestamp: Timestamp::now(),
            duration,
            status: ResponseStatus::Error {
                code,
                message: message.into(),
            },
            metadata: Arc::new(HashMap::new()),
        }
    }

    /// Create a cancelled response context
    pub fn cancelled(request_id: impl Into<String>, duration: std::time::Duration) -> Self {
        Self {
            request_id: request_id.into(),
            timestamp: Timestamp::now(),
            duration,
            status: ResponseStatus::Cancelled,
            metadata: Arc::new(HashMap::new()),
        }
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        Arc::make_mut(&mut self.metadata).insert(key.into(), value.into());
        self
    }

    /// Check if response is successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self.status, ResponseStatus::Success)
    }

    /// Check if response is an error
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.status, ResponseStatus::Error { .. })
    }

    /// Get error information if response is an error
    #[must_use]
    pub fn error_info(&self) -> Option<(i32, &str)> {
        match &self.status {
            ResponseStatus::Error { code, message } => Some((*code, message)),
            _ => None,
        }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::Error { code, message } => write!(f, "Error({code}: {message})"),
            Self::Partial => write!(f, "Partial"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

// ============================================================================
// Enhanced Client Management and Session Tracking
// ============================================================================

/// Client identification methods for enhanced request routing and analytics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientId {
    /// Explicit client ID from header
    Header(String),
    /// Bearer token from Authorization header
    Token(String),
    /// Session cookie
    Session(String),
    /// Query parameter
    QueryParam(String),
    /// Hash of User-Agent (fallback)
    UserAgent(String),
    /// Anonymous client
    Anonymous,
}

impl ClientId {
    /// Get the string representation of the client ID
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Header(id)
            | Self::Token(id)
            | Self::Session(id)
            | Self::QueryParam(id)
            | Self::UserAgent(id) => id,
            Self::Anonymous => "anonymous",
        }
    }

    /// Check if the client is authenticated
    #[must_use]
    pub const fn is_authenticated(&self) -> bool {
        matches!(self, Self::Token(_) | Self::Session(_))
    }

    /// Get the authentication method
    #[must_use]
    pub const fn auth_method(&self) -> &'static str {
        match self {
            Self::Header(_) => "header",
            Self::Token(_) => "bearer_token",
            Self::Session(_) => "session_cookie",
            Self::QueryParam(_) => "query_param",
            Self::UserAgent(_) => "user_agent",
            Self::Anonymous => "anonymous",
        }
    }
}

/// Client session information for tracking and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSession {
    /// Unique client identifier
    pub client_id: String,
    /// Client name (optional, human-readable)
    pub client_name: Option<String>,
    /// When the client connected
    pub connected_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Number of requests made
    pub request_count: usize,
    /// Transport type (stdio, http, websocket, etc.)
    pub transport_type: String,
    /// Authentication status
    pub authenticated: bool,
    /// Client capabilities (optional)
    pub capabilities: Option<serde_json::Value>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ClientSession {
    /// Create a new client session
    #[must_use]
    pub fn new(client_id: String, transport_type: String) -> Self {
        let now = Utc::now();
        Self {
            client_id,
            client_name: None,
            connected_at: now,
            last_activity: now,
            request_count: 0,
            transport_type,
            authenticated: false,
            capabilities: None,
            metadata: HashMap::new(),
        }
    }

    /// Update activity timestamp and increment request count
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
        self.request_count += 1;
    }

    /// Set authentication status and client info
    pub fn authenticate(&mut self, client_name: Option<String>) {
        self.authenticated = true;
        self.client_name = client_name;
    }

    /// Set client capabilities
    pub fn set_capabilities(&mut self, capabilities: serde_json::Value) {
        self.capabilities = Some(capabilities);
    }

    /// Get session duration
    #[must_use]
    pub fn session_duration(&self) -> chrono::Duration {
        self.last_activity - self.connected_at
    }

    /// Check if session is idle (no activity for specified duration)
    #[must_use]
    pub fn is_idle(&self, idle_threshold: chrono::Duration) -> bool {
        Utc::now() - self.last_activity > idle_threshold
    }
}

/// Request analytics information for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Client identifier
    pub client_id: String,
    /// Tool or method name
    pub method_name: String,
    /// Request parameters (sanitized for privacy)
    pub parameters: serde_json::Value,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// HTTP status code (if applicable)
    pub status_code: Option<u16>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RequestInfo {
    /// Create a new request info
    #[must_use]
    pub fn new(client_id: String, method_name: String, parameters: serde_json::Value) -> Self {
        Self {
            timestamp: Utc::now(),
            client_id,
            method_name,
            parameters,
            response_time_ms: None,
            success: false,
            error_message: None,
            status_code: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark the request as completed successfully
    #[must_use]
    pub const fn complete_success(mut self, response_time_ms: u64) -> Self {
        self.response_time_ms = Some(response_time_ms);
        self.success = true;
        self.status_code = Some(200);
        self
    }

    /// Mark the request as failed
    #[must_use]
    pub fn complete_error(mut self, response_time_ms: u64, error: String) -> Self {
        self.response_time_ms = Some(response_time_ms);
        self.success = false;
        self.error_message = Some(error);
        self.status_code = Some(500);
        self
    }

    /// Set HTTP status code
    #[must_use]
    pub const fn with_status_code(mut self, code: u16) -> Self {
        self.status_code = Some(code);
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Client identification extractor for various transport mechanisms
#[derive(Debug)]
pub struct ClientIdExtractor {
    /// Authentication tokens mapping token -> `client_id`
    auth_tokens: Arc<dashmap::DashMap<String, String>>,
}

impl ClientIdExtractor {
    /// Create a new client ID extractor
    #[must_use]
    pub fn new() -> Self {
        Self {
            auth_tokens: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Register an authentication token for a client
    pub fn register_token(&self, token: String, client_id: String) {
        self.auth_tokens.insert(token, client_id);
    }

    /// Remove an authentication token
    pub fn revoke_token(&self, token: &str) {
        self.auth_tokens.remove(token);
    }

    /// List all registered tokens (for admin purposes)
    #[must_use]
    pub fn list_tokens(&self) -> Vec<(String, String)> {
        self.auth_tokens
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Extract client ID from HTTP headers
    #[must_use]
    #[allow(clippy::significant_drop_tightening)]
    pub fn extract_from_http_headers(&self, headers: &HashMap<String, String>) -> ClientId {
        // 1. Check for explicit client ID header
        if let Some(client_id) = headers.get("x-client-id") {
            return ClientId::Header(client_id.clone());
        }

        // 2. Check for Authorization header with Bearer token
        if let Some(auth) = headers.get("authorization")
            && let Some(token) = auth.strip_prefix("Bearer ")
        {
            // Look up client ID from token
            let token_lookup = self.auth_tokens.iter().find(|e| e.key() == token);
            if let Some(entry) = token_lookup {
                let client_id = entry.value().clone();
                drop(entry); // Explicitly drop the lock guard early
                return ClientId::Token(client_id);
            }
            // Token not found - return the token itself as identifier
            return ClientId::Token(token.to_string());
        }

        // 3. Check for session cookie
        if let Some(cookie) = headers.get("cookie") {
            for cookie_part in cookie.split(';') {
                let parts: Vec<&str> = cookie_part.trim().splitn(2, '=').collect();
                if parts.len() == 2 && (parts[0] == "session_id" || parts[0] == "sessionid") {
                    return ClientId::Session(parts[1].to_string());
                }
            }
        }

        // 4. Use User-Agent hash as fallback
        if let Some(user_agent) = headers.get("user-agent") {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            user_agent.hash(&mut hasher);
            return ClientId::UserAgent(format!("ua_{:x}", hasher.finish()));
        }

        ClientId::Anonymous
    }

    /// Extract client ID from query parameters
    #[must_use]
    pub fn extract_from_query(&self, query_params: &HashMap<String, String>) -> Option<ClientId> {
        query_params
            .get("client_id")
            .map(|client_id| ClientId::QueryParam(client_id.clone()))
    }

    /// Extract client ID from multiple sources (with priority)
    #[must_use]
    pub fn extract_client_id(
        &self,
        headers: Option<&HashMap<String, String>>,
        query_params: Option<&HashMap<String, String>>,
    ) -> ClientId {
        // Try query parameters first (highest priority)
        if let Some(params) = query_params
            && let Some(client_id) = self.extract_from_query(params)
        {
            return client_id;
        }

        // Then try headers
        if let Some(headers) = headers {
            return self.extract_from_http_headers(headers);
        }

        ClientId::Anonymous
    }
}

impl Default for ClientIdExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait to add enhanced client management to `RequestContext`
pub trait RequestContextExt {
    /// Set client ID using `ClientId` enum
    #[must_use]
    fn with_enhanced_client_id(self, client_id: ClientId) -> Self;

    /// Extract and set client ID from headers and query params
    #[must_use]
    fn extract_client_id(
        self,
        extractor: &ClientIdExtractor,
        headers: Option<&HashMap<String, String>>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Self;

    /// Get the enhanced client ID
    fn get_enhanced_client_id(&self) -> Option<ClientId>;
}

impl RequestContextExt for RequestContext {
    fn with_enhanced_client_id(self, client_id: ClientId) -> Self {
        self.with_client_id(client_id.as_str())
            .with_metadata("client_id_method", client_id.auth_method())
            .with_metadata("client_authenticated", client_id.is_authenticated())
    }

    fn extract_client_id(
        self,
        extractor: &ClientIdExtractor,
        headers: Option<&HashMap<String, String>>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Self {
        let client_id = extractor.extract_client_id(headers, query_params);
        self.with_enhanced_client_id(client_id)
    }

    fn get_enhanced_client_id(&self) -> Option<ClientId> {
        self.client_id.as_ref().map(|id| {
            let method = self
                .get_metadata("client_id_method")
                .and_then(|v| v.as_str())
                .unwrap_or("header");

            match method {
                "bearer_token" => ClientId::Token(id.clone()),
                "session_cookie" => ClientId::Session(id.clone()),
                "query_param" => ClientId::QueryParam(id.clone()),
                "user_agent" => ClientId::UserAgent(id.clone()),
                "anonymous" => ClientId::Anonymous,
                _ => ClientId::Header(id.clone()), // Default to header for "header" and unknown methods
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_creation() {
        let ctx = RequestContext::new();
        assert!(!ctx.request_id.is_empty());
        assert!(ctx.user_id.is_none());
        assert!(ctx.elapsed() < std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_request_context_builder() {
        let ctx = RequestContext::new()
            .with_user_id("user123")
            .with_session_id("session456")
            .with_metadata("key", "value");

        assert_eq!(ctx.user_id, Some("user123".to_string()));
        assert_eq!(ctx.session_id, Some("session456".to_string()));
        assert_eq!(
            ctx.get_metadata("key"),
            Some(&serde_json::Value::String("value".to_string()))
        );
    }

    #[test]
    fn test_response_context_creation() {
        let duration = std::time::Duration::from_millis(100);

        let success_ctx = ResponseContext::success("req1", duration);
        assert!(success_ctx.is_success());
        assert!(!success_ctx.is_error());

        let error_ctx = ResponseContext::error("req2", duration, 500, "Internal error");
        assert!(!error_ctx.is_success());
        assert!(error_ctx.is_error());
        assert_eq!(error_ctx.error_info(), Some((500, "Internal error")));
    }

    #[test]
    fn test_context_derivation() {
        let parent_ctx = RequestContext::new()
            .with_user_id("user123")
            .with_metadata("key", "value");

        let child_ctx = parent_ctx.derive();

        // Should have new request ID
        assert_ne!(parent_ctx.request_id, child_ctx.request_id);

        // Should inherit user info and metadata
        assert_eq!(parent_ctx.user_id, child_ctx.user_id);
        assert_eq!(
            parent_ctx.get_metadata("key"),
            child_ctx.get_metadata("key")
        );
    }

    // Tests for enhanced client management

    #[test]
    fn test_client_id_extraction() {
        let extractor = ClientIdExtractor::new();

        // Test header extraction
        let mut headers = HashMap::new();
        headers.insert("x-client-id".to_string(), "test-client".to_string());

        let client_id = extractor.extract_from_http_headers(&headers);
        assert_eq!(client_id, ClientId::Header("test-client".to_string()));
        assert_eq!(client_id.as_str(), "test-client");
        assert_eq!(client_id.auth_method(), "header");
        assert!(!client_id.is_authenticated());
    }

    #[test]
    fn test_bearer_token_extraction() {
        let extractor = ClientIdExtractor::new();
        extractor.register_token("token123".to_string(), "client-1".to_string());

        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer token123".to_string());

        let client_id = extractor.extract_from_http_headers(&headers);
        assert_eq!(client_id, ClientId::Token("client-1".to_string()));
        assert!(client_id.is_authenticated());
        assert_eq!(client_id.auth_method(), "bearer_token");
    }

    #[test]
    fn test_session_cookie_extraction() {
        let extractor = ClientIdExtractor::new();

        let mut headers = HashMap::new();
        headers.insert(
            "cookie".to_string(),
            "session_id=sess123; other=value".to_string(),
        );

        let client_id = extractor.extract_from_http_headers(&headers);
        assert_eq!(client_id, ClientId::Session("sess123".to_string()));
        assert!(client_id.is_authenticated());
    }

    #[test]
    fn test_user_agent_fallback() {
        let extractor = ClientIdExtractor::new();

        let mut headers = HashMap::new();
        headers.insert("user-agent".to_string(), "TestAgent/1.0".to_string());

        let client_id = extractor.extract_from_http_headers(&headers);
        if let ClientId::UserAgent(id) = client_id {
            assert!(id.starts_with("ua_"));
        } else {
            // Ensure test failure without panicking in production codepaths
            assert!(
                matches!(client_id, ClientId::UserAgent(_)),
                "Expected UserAgent ClientId"
            );
        }
    }

    #[test]
    fn test_client_session() {
        let mut session = ClientSession::new("test-client".to_string(), "http".to_string());
        assert!(!session.authenticated);
        assert_eq!(session.request_count, 0);

        session.update_activity();
        assert_eq!(session.request_count, 1);

        session.authenticate(Some("Test Client".to_string()));
        assert!(session.authenticated);
        assert_eq!(session.client_name, Some("Test Client".to_string()));

        // Test idle detection
        assert!(!session.is_idle(chrono::Duration::seconds(1)));
    }

    #[test]
    fn test_request_info() {
        let params = serde_json::json!({"param": "value"});
        let request = RequestInfo::new("client-1".to_string(), "test_method".to_string(), params);

        assert!(!request.success);
        assert!(request.response_time_ms.is_none());

        let completed = request.complete_success(150);
        assert!(completed.success);
        assert_eq!(completed.response_time_ms, Some(150));
        assert_eq!(completed.status_code, Some(200));
    }

    #[test]
    fn test_request_context_ext() {
        let extractor = ClientIdExtractor::new();

        let mut headers = HashMap::new();
        headers.insert("x-client-id".to_string(), "test-client".to_string());

        let ctx = RequestContext::new().extract_client_id(&extractor, Some(&headers), None);

        assert_eq!(ctx.client_id, Some("test-client".to_string()));
        assert_eq!(
            ctx.get_metadata("client_id_method"),
            Some(&serde_json::Value::String("header".to_string()))
        );
        assert_eq!(
            ctx.get_metadata("client_authenticated"),
            Some(&serde_json::Value::Bool(false))
        );

        let enhanced_id = ctx.get_enhanced_client_id();
        assert_eq!(
            enhanced_id,
            Some(ClientId::Header("test-client".to_string()))
        );
    }

    #[test]
    fn test_request_analytics() {
        let start = std::time::Instant::now();
        let request = RequestInfo::new(
            "client-123".to_string(),
            "get_data".to_string(),
            serde_json::json!({"filter": "active"}),
        );

        let response_time = start.elapsed().as_millis() as u64;
        let completed = request
            .complete_success(response_time)
            .with_metadata("cache_hit".to_string(), serde_json::json!(true));

        assert!(completed.success);
        assert!(completed.response_time_ms.is_some());
        assert_eq!(
            completed.metadata.get("cache_hit"),
            Some(&serde_json::json!(true))
        );
    }
}
