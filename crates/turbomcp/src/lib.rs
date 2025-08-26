//! # TurboMCP - Model Context Protocol SDK
//!
//! Rust SDK for the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/)
//! with SIMD acceleration, robust transport layer, graceful shutdown, and ergonomic APIs.
//!
//! ## Features
//!
//! - **MCP 2025-06-18 Specification** - Full compliance with latest protocol
//! - **SIMD-Accelerated JSON** - `simd-json` and `sonic-rs` for fast processing  
//! - **Multi-Transport** - STDIO, TCP, Unix sockets with runtime selection
//! - **Robust** - Circuit breakers, retry logic, graceful shutdown
//! - **Zero-Overhead Macros** - Ergonomic `#[server]`, `#[tool]`, `#[resource]` attributes
//! - **Context Injection** - Dependency injection and observability
//! - **Type Safety** - Compile-time validation with automatic schema generation
//!
//! ## Quick Start
//!
//! ```rust
//! use turbomcp::prelude::*;
//!
//! #[derive(Clone)]
//! struct Calculator {
//!     operations: std::sync::Arc<std::sync::atomic::AtomicU64>,
//! }
//!
//! #[server]
//! impl Calculator {
//!     #[tool("Add two numbers")]
//!     async fn add(&self, ctx: Context, a: i32, b: i32) -> McpResult<i32> {
//!         // Context injection provides automatic:
//!         // - Request correlation and distributed tracing
//!         // - Structured logging with metadata
//!         // - Performance monitoring and metrics
//!         ctx.info(&format!("Adding {} + {}", a, b)).await?;
//!         
//!         self.operations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//!         Ok(a + b)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = Calculator {
//!         operations: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
//!     };
//!     server.run_stdio().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Graceful Shutdown
//!
//! TurboMCP provides graceful shutdown for all transport methods:
//!
//! ```no_run
//! use turbomcp::prelude::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct Calculator {
//!     operations: Arc<std::sync::atomic::AtomicU64>,
//! }
//!
//! #[server]
//! impl Calculator {
//!     #[tool("Add numbers")]
//!     async fn add(&self, a: i32, b: i32) -> McpResult<i32> {
//!         Ok(a + b)
//!     }
//! }
//!
//! // Example: Get shutdown handle for graceful termination
//! let server = Calculator {
//!     operations: Arc::new(std::sync::atomic::AtomicU64::new(0)),
//! };
//!
//! // This gives you a handle to gracefully shutdown the server
//! let (server, shutdown_handle) = server.into_server_with_shutdown().unwrap();
//!
//! // In production, you'd spawn the server and wait for signals:
//! // tokio::spawn(async move { server.run_stdio().await });
//! // signal::ctrl_c().await?;
//! // shutdown_handle.shutdown().await;
//! ```
//!
//! ## Runtime Transport Selection
//!
//! ```no_run
//! use turbomcp::prelude::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct Calculator {
//!     operations: Arc<std::sync::atomic::AtomicU64>,
//! }
//!
//! #[server]
//! impl Calculator {
//!     #[tool("Add numbers")]
//!     async fn add(&self, a: i32, b: i32) -> McpResult<i32> {
//!         Ok(a + b)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = Calculator {
//!         operations: Arc::new(std::sync::atomic::AtomicU64::new(0)),
//!     };
//!     
//!     // Runtime transport selection based on environment
//!     match std::env::var("TRANSPORT").as_deref() {
//!         Ok("tcp") => {
//!             let port: u16 = std::env::var("PORT")
//!                 .unwrap_or_else(|_| "8080".to_string())
//!                 .parse()
//!                 .unwrap_or(8080);
//!             server.run_tcp(format!("127.0.0.1:{}", port)).await?;
//!         }
//!         Ok("unix") => {
//!             let path = std::env::var("SOCKET_PATH")
//!                 .unwrap_or_else(|_| "/tmp/mcp.sock".to_string());
//!             server.run_unix(path).await?;
//!         }
//!         _ => {
//!             server.run_stdio().await?;
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! TurboMCP provides ergonomic error handling with the `mcp_error!` macro:
//!
//! ```rust
//! use turbomcp::prelude::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct Calculator {
//!     operations: Arc<std::sync::atomic::AtomicU64>,
//! }
//!
//! #[server]
//! impl Calculator {
//!     #[tool("Divide two numbers")]
//!     async fn divide(&self, a: f64, b: f64) -> McpResult<f64> {
//!         if b == 0.0 {
//!             return Err(turbomcp::McpError::Tool("Cannot divide by zero".to_string()));
//!         }
//!         Ok(a / b)
//!     }
//! }
//! ```
//!
//! ## Advanced Features
//!
//! TurboMCP supports resources and prompts alongside tools:
//!
//! ```rust
//! use turbomcp::prelude::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct Calculator {
//!     operations: Arc<std::sync::atomic::AtomicU64>,
//! }
//!
//! #[server]
//! impl Calculator {
//!     #[tool("Add numbers")]
//!     async fn add(&self, a: i32, b: i32) -> McpResult<i32> {
//!         Ok(a + b)
//!     }
//!
//!     #[resource("calc://history")]
//!     async fn history(&self, _uri: String) -> McpResult<String> {
//!         Ok("Calculation history data".to_string())
//!     }
//!     
//!     #[prompt("Generate calculation report for {operation}")]
//!     async fn calc_report(&self, operation: String) -> McpResult<String> {
//!         Ok(format!("Report for {operation} operations"))
//!     }
//! }
//! ```
//!
//! ## Feature-Gated Transports
//!
//! Reduce binary size by selecting only the transports you need:
//!
//! ```toml
//! # Cargo.toml - TCP-only server (no STDIO)
//! [dependencies]
//! turbomcp = { version = "1.0", default-features = false, features = ["tcp"] }
//! ```
//!
//! Available feature combinations:
//! - `minimal` - Just STDIO (works everywhere)
//! - `network` - STDIO + TCP
//! - `server-only` - TCP + Unix (no STDIO)
//! - `all-transports` - Maximum flexibility
//!
//! For more examples and advanced usage, see the [examples directory](https://github.com/Epistates/turbomcp/tree/main/crates/turbomcp/examples).
//!
//! ## Architecture
//!
//! - **MCP 2025-06-18 Specification** - Full protocol compliance
//! - **Multi-Transport Support** - STDIO, TCP, Unix sockets with runtime selection
//! - **Graceful Shutdown** - Lifecycle management
//! - **Zero-Overhead Macros** - Ergonomic `#[server]`, `#[tool]`, `#[resource]` attributes
//! - **Type Safety** - Compile-time validation and automatic schema generation
//! - **SIMD Acceleration** - High-throughput JSON processing

#![deny(missing_docs)]
#![warn(clippy::all)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,  // Too pedantic for library APIs
    clippy::return_self_not_must_use,  // Constructor methods don't need must_use
    clippy::struct_excessive_bools,  // Sometimes bools are the right design
    clippy::missing_panics_doc,  // Panic docs added where genuinely needed
    clippy::default_trait_access,  // Default::default() is sometimes clearer
    clippy::missing_const_for_fn,  // Const fn where it makes sense, not everywhere
    clippy::use_self,  // Sometimes explicit types are clearer
    clippy::uninlined_format_args  // Sometimes variables in format! are clearer
)]

use std::collections::HashMap;
use std::sync::Arc;

// async_trait re-exported below
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// Re-export core types for convenience
pub use turbomcp_core::{MessageId, RequestContext};
// Re-export key protocol types (avoiding * import to prevent ambiguous re-exports)
pub use turbomcp_protocol::GetPromptResult;
pub use turbomcp_protocol::jsonrpc::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
pub use turbomcp_protocol::types::{
    CallToolRequest, CallToolResult, ClientCapabilities, Content, ImageContent, Implementation,
    InitializeRequest, InitializeResult, PromptMessage, Resource, ServerCapabilities, TextContent,
    Tool, ToolInputSchema,
};
pub use turbomcp_server::{
    McpServer, McpServer as Server, ServerBuilder, ServerError, ServerResult, ShutdownHandle,
    handlers,
};

// Re-export async_trait for macros
pub use async_trait::async_trait;

// Core TurboMCP modules
pub mod auth;
pub mod context;
pub mod context_factory;
pub mod elicitation;
pub mod helpers;

pub mod injection;
pub mod lifespan;
pub mod progress;
pub mod registry;
pub mod router;
pub mod server;
pub mod session;
pub mod simd;
pub mod sse_server;
pub mod structured;
#[cfg(test)]
pub mod test_utils;
pub mod transport;
pub mod validation;

#[cfg(feature = "uri-templates")]
pub mod uri;

#[cfg(feature = "schema-generation")]
pub mod schema;

// Re-export from submodules
// Note: auth and session both define SessionConfig, so we rename one to avoid ambiguous re-exports
pub use crate::auth::SessionConfig as AuthSessionConfig;
pub use crate::auth::{
    ApiKeyProvider, AuthConfig, AuthContext, AuthCredentials, AuthManager, AuthMiddleware,
    AuthProvider, AuthProviderConfig, AuthProviderType, OAuth2Config, OAuth2FlowType,
    OAuth2Provider, TokenInfo, UserInfo,
};
pub use crate::context::*;
pub use crate::context_factory::{
    ContextCreationStrategy, ContextFactory, ContextFactoryConfig, ContextFactoryProvider,
    CorrelationId, RequestScope,
};
pub use crate::elicitation::*;
pub use crate::helpers::*;
pub use crate::injection::*;
pub use crate::lifespan::*;
pub use crate::progress::*;
pub use crate::registry::*;
pub use crate::router::{ToolRouter, ToolRouterExt};
pub use crate::server::*;
pub use crate::session::*;
pub use crate::simd::*;
pub use crate::sse_server::*;
pub use crate::structured::*;
pub use crate::transport::*;
pub use crate::validation::*;

// Re-export inventory for macro use
pub use inventory;

// Re-export macros
pub use turbomcp_macros::{mcp_error, mcp_text, prompt, resource, server, tool, tool_result};

/// Convenient prelude for `TurboMCP` applications
pub mod prelude {
    // Re-export procedural macros for zero-boilerplate development
    pub use super::{mcp_error, mcp_text, prompt, resource, server, tool, tool_result};

    pub use super::{
        ApiKeyProvider, AuthConfig, AuthContext, AuthCredentials, AuthManager, AuthMiddleware,
        AuthProvider, AuthProviderConfig, AuthProviderType, CallToolRequest, CallToolResult,
        Context, ElicitationManager, HandlerMetadata, HandlerRegistration, McpError, McpResult,
        McpServer, OAuth2Config, OAuth2FlowType, OAuth2Provider, RequestContext, Server,
        ServerBuilder, ServerError, TokenInfo, Transport, TransportConfig, TransportFactory,
        TransportManager, TurboMcpServer, UserInfo, error_text, handlers, prompt_result,
        resource_result, text, tool_error, tool_success,
    };

    // Re-export essential types
    pub use turbomcp_protocol::types::{
        Content, GetPromptResult, Prompt, ReadResourceResult, Resource, TextContent, Tool,
    };

    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
}

/// `TurboMCP` result type
pub type McpResult<T> = Result<T, McpError>;

/// `TurboMCP` error type
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// Server error
    #[error("Server error: {0}")]
    Server(#[from] turbomcp_server::ServerError),

    /// Protocol error  
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Tool execution error
    #[error("Tool error: {0}")]
    Tool(String),

    /// Resource access error
    #[error("Resource error: {0}")]
    Resource(String),

    /// Prompt processing error
    #[error("Prompt error: {0}")]
    Prompt(String),

    /// Context error
    #[error("Context error: {0}")]
    Context(String),

    /// Authorization/authentication error
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Network/connectivity error
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Schema generation error
    #[error("Schema error: {0}")]
    Schema(String),

    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Internal error - for backwards compatibility
    #[error("Internal error: {0}")]
    Internal(String),

    /// Invalid request error
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl McpError {
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create an invalid request error
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    /// Create a tool error
    pub fn tool(msg: impl Into<String>) -> Self {
        Self::Tool(msg.into())
    }

    /// Create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::Protocol(msg.into())
    }

    /// Create a resource error
    pub fn resource(msg: impl Into<String>) -> Self {
        Self::Resource(msg.into())
    }

    /// Create an unauthorized error
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create an invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
}

impl From<turbomcp_transport::core::TransportError> for McpError {
    fn from(err: turbomcp_transport::core::TransportError) -> Self {
        Self::Transport(err.to_string())
    }
}

impl Clone for McpError {
    fn clone(&self) -> Self {
        match self {
            Self::Server(e) => {
                // Convert the server error to string to avoid any complex cloning issues
                let error_msg = format!("{e}");
                Self::Server(turbomcp_server::ServerError::Internal(error_msg))
            }
            Self::Protocol(s) => Self::Protocol(s.clone()),
            Self::Tool(s) => Self::Tool(s.clone()),
            Self::Resource(s) => Self::Resource(s.clone()),
            Self::Prompt(s) => Self::Prompt(s.clone()),
            Self::Context(s) => Self::Context(s.clone()),
            Self::Unauthorized(s) => Self::Unauthorized(s.clone()),
            Self::Network(s) => Self::Network(s.clone()),
            Self::InvalidInput(s) => Self::InvalidInput(s.clone()),
            Self::Schema(s) => Self::Schema(s.clone()),
            Self::Transport(s) => Self::Transport(s.clone()),
            Self::Internal(s) => Self::Internal(s.clone()),
            Self::InvalidRequest(s) => Self::InvalidRequest(s.clone()),
            Self::Serialization(e) => {
                // Convert the serialization error to string to avoid cloning complexity
                let error_msg = format!("{e}");
                let io_error = std::io::Error::other(error_msg);
                Self::Serialization(serde_json::Error::io(io_error))
            }
        }
    }
}

/// TurboMCP server trait for ergonomic server definition
#[async_trait::async_trait]
pub trait TurboMcpServer: Send + Sync + 'static + HandlerRegistration {
    /// Get server name
    fn name(&self) -> &'static str {
        "TurboMCP Server"
    }

    /// Get server version
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get server description
    fn description(&self) -> Option<&str> {
        None
    }

    /// Server initialization hook
    async fn startup(&self) -> McpResult<()> {
        Ok(())
    }

    /// Server shutdown hook  
    async fn shutdown(&self) -> McpResult<()> {
        Ok(())
    }

    /// Run server with STDIO transport
    /// Note: Disabled due to async trait lifetime constraints
    /*
    async fn run_stdio(self: Arc<Self>) -> McpResult<()> {
        // Initialize server
        self.startup().await?;

        // Build and run MCP server
        let server = self.build_server().await?;
        let result = server.run_stdio().await;

        // Cleanup
        self.shutdown().await?;

        Ok(result?)
    }
    */
    /// Build the underlying MCP server
    async fn build_server(&self) -> McpResult<McpServer> {
        let mut builder = ServerBuilder::new()
            .name(self.name())
            .version(self.version());

        if let Some(desc) = self.description() {
            builder = builder.description(desc);
        }

        // Register handlers
        self.register_with_builder(&mut builder).await?;

        Ok(builder.build())
    }
}

/// Context for `TurboMCP` handlers with dependency injection
#[derive(Clone)]
pub struct Context {
    /// Request context from MCP core
    pub request: RequestContext,
    /// Server-specific data
    pub data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Handler metadata
    pub handler: HandlerMetadata,
    /// Dependency injection container
    pub container: context::Container,
}

/// Metadata about the current handler
#[derive(Debug, Clone)]
pub struct HandlerMetadata {
    /// Handler name
    pub name: String,
    /// Handler type (tool, prompt, resource)
    pub handler_type: String,
    /// Handler description
    pub description: Option<String>,
}

impl Context {
    /// Create a new context
    #[must_use]
    pub fn new(request: RequestContext, handler: HandlerMetadata) -> Self {
        Self {
            request,
            data: Arc::new(RwLock::new(HashMap::new())),
            handler,
            container: context::Container::new(),
        }
    }

    /// Create a new context with a shared container
    #[must_use]
    pub fn with_container(
        request: RequestContext,
        handler: HandlerMetadata,
        container: context::Container,
    ) -> Self {
        Self {
            request,
            data: Arc::new(RwLock::new(HashMap::new())),
            handler,
            container,
        }
    }

    /// Resolve a service from the dependency injection container
    pub async fn resolve<T: 'static + Clone>(&self, name: &str) -> McpResult<T> {
        self.container.resolve_with_dependencies(name).await
    }

    /// Resolve a service by type name (convenience method)
    pub async fn resolve_by_type<T: 'static + Clone>(&self) -> McpResult<T> {
        let type_name = std::any::type_name::<T>();
        self.resolve(type_name).await
    }

    /// Register a service with the container
    pub async fn register<T: 'static + Send + Sync>(&self, name: &str, service: T) {
        self.container.register(name, service).await;
    }

    /// Register a singleton factory with the container
    pub async fn register_singleton<F, T>(&self, name: &str, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
        T: Send + Sync + Clone + 'static,
    {
        self.container.register_singleton(name, factory).await;
    }

    /// Log an info message to the client
    pub async fn info<S: AsRef<str>>(&self, message: S) -> McpResult<()> {
        tracing::info!("{}", message.as_ref());
        // Logging notification sent via tracing infrastructure
        Ok(())
    }

    /// Log a warning message to the client
    pub async fn warn<S: AsRef<str>>(&self, message: S) -> McpResult<()> {
        tracing::warn!("{}", message.as_ref());
        // Logging notification sent via tracing infrastructure
        Ok(())
    }

    /// Log an error message to the client
    pub async fn error<S: AsRef<str>>(&self, message: S) -> McpResult<()> {
        tracing::error!("{}", message.as_ref());
        // Logging notification sent via tracing infrastructure
        Ok(())
    }

    /// Report progress for long-running operations
    pub async fn report_progress(&self, progress: f64, total: Option<f64>) -> McpResult<()> {
        tracing::debug!("Progress: {} / {:?}", progress, total);

        // Generate or use existing progress token
        let token = crate::progress::ProgressToken::new();

        // Update progress using the global progress manager
        crate::progress::global_progress_manager().update_progress(&token, progress, total)?;

        // Progress notification sent to MCP client via notification system
        // Integrated with the MCP notification protocol

        Ok(())
    }

    /// Store data in context
    pub async fn set<T: Serialize>(&self, key: &str, value: T) -> McpResult<()> {
        let json_value = serde_json::to_value(value)?;
        self.data.write().await.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Retrieve data from context
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> McpResult<Option<T>> {
        if let Some(value) = self.data.read().await.get(key) {
            Ok(Some(serde_json::from_value(value.clone())?))
        } else {
            Ok(None)
        }
    }
}

/// Helper trait for handler registration
#[async_trait::async_trait]
pub trait HandlerRegistration {
    /// Register with a server builder
    async fn register_with_builder(&self, builder: &mut ServerBuilder) -> McpResult<()>;
}

/// Handler type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerType {
    /// Tool handler
    Tool,
    /// Prompt handler  
    Prompt,
    /// Resource handler
    Resource,
}

/// Handler registration information
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    /// Handler name
    pub name: String,
    /// Handler type
    pub handler_type: HandlerType,
    /// Handler description
    pub description: Option<String>,
    /// Handler schema
    pub schema: Option<serde_json::Value>,
}

// The default server implementation will be generated by the #[server] macro
