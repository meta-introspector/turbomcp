//! # MCP Protocol Types
//!
//! This module contains all the type definitions for the Model Context Protocol
//! according to the 2025-06-18 specification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use turbomcp_core::MessageId;

/// Protocol version string
pub type ProtocolVersion = String;

/// JSON-RPC request identifier
pub type RequestId = MessageId;

/// Progress token for tracking long-running operations
pub type ProgressToken = String;

/// URI string
pub type Uri = String;

/// MIME type
pub type MimeType = String;

/// Base64 encoded data
pub type Base64String = String;

// ============================================================================
// JSON-RPC Error Codes
// ============================================================================

/// Standard JSON-RPC error codes per specification
pub mod error_codes {
    /// Parse error - Invalid JSON was received by the server
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid Request - The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found - The method does not exist / is not available
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params - Invalid method parameter(s)
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error - Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// JSON-RPC error structure per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    /// The error type that occurred
    pub code: i32,
    /// A short description of the error (should be limited to a concise single sentence)
    pub message: String,
    /// Additional information about the error (detailed error information, nested errors, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    /// Create a new JSON-RPC error with additional data
    pub fn with_data(code: i32, message: String, data: serde_json::Value) -> Self {
        Self {
            code,
            message,
            data: Some(data),
        }
    }

    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(error_codes::PARSE_ERROR, "Parse error".to_string())
    }

    /// Create an invalid request error
    pub fn invalid_request() -> Self {
        Self::new(error_codes::INVALID_REQUEST, "Invalid Request".to_string())
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {method}"),
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(details: &str) -> Self {
        Self::new(
            error_codes::INVALID_PARAMS,
            format!("Invalid params: {details}"),
        )
    }

    /// Create an internal error
    pub fn internal_error(details: &str) -> Self {
        Self::new(
            error_codes::INTERNAL_ERROR,
            format!("Internal error: {details}"),
        )
    }
}

/// Cursor for pagination
pub type Cursor = String;

// ============================================================================
// Base Metadata Interface
// ============================================================================

/// Base interface for metadata with name (identifier) and title (display name) properties.
/// Per MCP specification 2025-06-18, this is the foundation for Tool, Resource, and Prompt metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMetadata {
    /// Intended for programmatic or logical use, but used as a display name in past specs or fallback (if title isn't present).
    pub name: String,

    /// Intended for UI and end-user contexts — optimized to be human-readable and easily understood,
    /// even by those unfamiliar with domain-specific terminology.
    ///
    /// If not provided, the name should be used for display (except for Tool,
    /// where `annotations.title` should be given precedence over using `name`, if present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Implementation information for MCP clients and servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    /// Implementation name
    pub name: String,
    /// Implementation display title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Implementation version
    pub version: String,
}

/// General annotations that can be attached to various MCP objects
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Annotations {
    /// Audience-specific hints or information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    /// Priority level for ordering or importance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    /// Additional custom annotations
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Core Protocol Types
// ============================================================================

/// Client-initiated request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum ClientRequest {
    /// Initialize the connection
    #[serde(rename = "initialize")]
    Initialize(InitializeRequest),

    /// List available tools
    #[serde(rename = "tools/list")]
    ListTools(ListToolsRequest),

    /// Call a tool
    #[serde(rename = "tools/call")]
    CallTool(CallToolRequest),

    /// List available prompts
    #[serde(rename = "prompts/list")]
    ListPrompts(ListPromptsRequest),

    /// Get a specific prompt
    #[serde(rename = "prompts/get")]
    GetPrompt(GetPromptRequest),

    /// List available resources
    #[serde(rename = "resources/list")]
    ListResources(ListResourcesRequest),

    /// Read a resource
    #[serde(rename = "resources/read")]
    ReadResource(ReadResourceRequest),

    /// Subscribe to resource updates
    #[serde(rename = "resources/subscribe")]
    Subscribe(SubscribeRequest),

    /// Unsubscribe from resource updates
    #[serde(rename = "resources/unsubscribe")]
    Unsubscribe(UnsubscribeRequest),

    /// Set logging level
    #[serde(rename = "logging/setLevel")]
    SetLevel(SetLevelRequest),

    /// Create a message (sampling)
    #[serde(rename = "sampling/createMessage")]
    CreateMessage(CreateMessageRequest),

    /// List filesystem roots
    #[serde(rename = "roots/list")]
    ListRoots(ListRootsRequest),
}

/// Server-initiated request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum ServerRequest {
    /// Ping to check connection
    #[serde(rename = "ping")]
    Ping,
}

/// Client-initiated notification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum ClientNotification {
    /// Connection initialized
    #[serde(rename = "notifications/initialized")]
    Initialized(InitializedNotification),

    /// Progress update
    #[serde(rename = "notifications/progress")]
    Progress(ProgressNotification),

    /// Roots list changed
    #[serde(rename = "notifications/roots/list_changed")]
    RootsListChanged(RootsListChangedNotification),
}

/// Server-initiated notification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum ServerNotification {
    /// Log message
    #[serde(rename = "notifications/message")]
    Message(LoggingNotification),

    /// Resource updated
    #[serde(rename = "notifications/resources/updated")]
    ResourceUpdated(ResourceUpdatedNotification),

    /// Resource list changed
    #[serde(rename = "notifications/resources/list_changed")]
    ResourceListChanged,

    /// Progress update
    #[serde(rename = "notifications/progress")]
    Progress(ProgressNotification),

    /// Request cancellation
    #[serde(rename = "notifications/cancelled")]
    Cancelled(CancelledNotification),

    /// Prompts list changed
    #[serde(rename = "notifications/prompts/list_changed")]
    PromptsListChanged,

    /// Tools list changed
    #[serde(rename = "notifications/tools/list_changed")]
    ToolsListChanged,

    /// Roots list changed
    #[serde(rename = "notifications/roots/list_changed")]
    RootsListChanged,
}

// ============================================================================
// Initialization
// ============================================================================

/// Initialize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: ProtocolVersion,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client implementation info
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
}

/// Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: ProtocolVersion,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server implementation info
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
    /// Additional instructions for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Initialized notification (no parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializedNotification;

// ============================================================================
// Capabilities
// ============================================================================

/// Client capabilities per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    /// Experimental, non-standard capabilities that the client supports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,

    /// Present if the client supports listing roots
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapabilities>,

    /// Present if the client supports sampling from an LLM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapabilities>,

    /// Present if the client supports elicitation from the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<ElicitationCapabilities>,
}

/// Server capabilities per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    /// Experimental, non-standard capabilities that the server supports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,

    /// Present if the server supports sending log messages to the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapabilities>,

    /// Present if the server supports argument autocompletion suggestions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<CompletionCapabilities>,

    /// Present if the server offers any prompt templates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapabilities>,

    /// Present if the server offers any resources to read
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapabilities>,

    /// Present if the server offers any tools to call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapabilities>,
}

/// Sampling capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SamplingCapabilities;

/// Elicitation capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElicitationCapabilities;

/// Completion capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionCapabilities;

/// Roots capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RootsCapabilities {
    /// Whether list can change
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Logging capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingCapabilities;

/// Prompts capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptsCapabilities {
    /// Whether list can change
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcesCapabilities {
    /// Whether subscribe is supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,

    /// Whether list can change
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Tools capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsCapabilities {
    /// Whether list can change
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

// ============================================================================
// Content Types
// ============================================================================

/// Content block union type per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text(TextContent),
    /// Image content
    #[serde(rename = "image")]
    Image(ImageContent),
    /// Audio content
    #[serde(rename = "audio")]
    Audio(AudioContent),
    /// Resource link
    #[serde(rename = "resource_link")]
    ResourceLink(ResourceLink),
    /// Embedded resource
    #[serde(rename = "resource")]
    Resource(EmbeddedResource),
}

/// Compatibility alias for the old Content enum
pub type Content = ContentBlock;

/// Text content per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    /// The text content of the message
    pub text: String,
    /// Optional annotations for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Image content per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    /// The base64-encoded image data
    pub data: String,
    /// The MIME type of the image. Different providers may support different image types
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    /// Optional annotations for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Audio content per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioContent {
    /// The base64-encoded audio data
    pub data: String,
    /// The MIME type of the audio. Different providers may support different audio types
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    /// Optional annotations for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Resource link per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLink {
    /// Resource name (programmatic identifier)
    pub name: String,
    /// Display title for UI contexts (optional, falls back to name if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The URI of this resource
    pub uri: String,
    /// A description of what this resource represents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The MIME type of this resource, if known
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Optional annotations for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// The size of the raw resource content, if known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Embedded resource content per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedResource {
    /// The embedded resource content (text or binary)
    pub resource: ResourceContent,
    /// Optional annotations for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Role in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User role
    User,
    /// Assistant role
    Assistant,
}

// ============================================================================
// Tool Types
// ============================================================================

/// Tool-specific annotations for additional tool information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolAnnotations {
    /// Title for display purposes - takes precedence over name for UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Audience-specific information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    /// Priority for ordering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    /// Additional custom annotations
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Tool definition per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name (programmatic identifier)
    pub name: String,

    /// Display title for UI contexts (optional, falls back to name if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Human-readable description of the tool
    /// This can be used by clients to improve the LLM's understanding of available tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema object defining the expected parameters for the tool
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,

    /// Optional JSON Schema object defining the structure of the tool's output
    /// returned in the structuredContent field of a CallToolResult
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<ToolOutputSchema>,

    /// Optional additional tool information
    /// Display name precedence order is: title, annotations.title, then name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,

    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Tool input schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    /// Must be "object" for tool input schemas
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Schema properties defining the tool parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// List of required property names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Whether additional properties are allowed
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<bool>,
}

/// Tool output schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutputSchema {
    /// Must be "object" for tool output schemas
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Schema properties defining the tool output structure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// List of required property names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Whether additional properties are allowed
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<bool>,
}

/// List tools request (no parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsRequest;

/// List tools result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// Available tools
    pub tools: Vec<Tool>,
    /// Optional continuation token
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Call tool request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    /// Tool name
    pub name: String,
    /// Tool arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, serde_json::Value>>,
}

/// Call tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Result content
    pub content: Vec<ContentBlock>,
    /// Whether the operation failed
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

// ============================================================================
// Prompt Types
// ============================================================================

/// Prompt definition per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Prompt name (programmatic identifier)
    pub name: String,

    /// Display title for UI contexts (optional, falls back to name if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// An optional description of what this prompt provides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A list of arguments to use for templating the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,

    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Prompt argument definition per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name (programmatic identifier)
    pub name: String,

    /// Display title for UI contexts (optional, falls back to name if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// A human-readable description of the argument
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether this argument must be provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Prompt input parameters
pub type PromptInput = HashMap<String, serde_json::Value>;

/// List prompts request (no parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsRequest;

/// List prompts result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResult {
    /// Available prompts
    pub prompts: Vec<Prompt>,
    /// Optional continuation token
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Get prompt request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptRequest {
    /// Prompt name
    pub name: String,
    /// Prompt arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<PromptInput>,
}

/// Get prompt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    /// Prompt description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt messages
    pub messages: Vec<PromptMessage>,
}

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Message role
    pub role: Role,
    /// Message content
    pub content: Content,
}

// ============================================================================
// Resource Types
// ============================================================================

/// Resource definition per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource name (programmatic identifier)
    pub name: String,

    /// Display title for UI contexts (optional, falls back to name if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The URI of this resource
    pub uri: String,

    /// A description of what this resource represents
    /// This can be used by clients to improve the LLM's understanding of available resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The MIME type of this resource, if known
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Optional annotations for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,

    /// The size of the raw resource content, in bytes (before base64 encoding or tokenization), if known
    /// This can be used by Hosts to display file sizes and estimate context window usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Base resource contents interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContents {
    /// The URI of this resource
    pub uri: String,
    /// The MIME type of this resource, if known
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Text resource contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextResourceContents {
    /// The URI of this resource
    pub uri: String,
    /// The MIME type of this resource, if known
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// The text content (must only be set for text-representable data)
    pub text: String,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Binary resource contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobResourceContents {
    /// The URI of this resource
    pub uri: String,
    /// The MIME type of this resource, if known
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Base64-encoded binary data
    pub blob: String,
    /// General metadata field for extensions and custom data
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Union type for resource contents (text or binary)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceContent {
    /// Text resource content
    Text(TextResourceContents),
    /// Binary resource content
    Blob(BlobResourceContents),
}

/// List resources request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// List resources result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// Available resources
    pub resources: Vec<Resource>,
    /// Optional continuation token
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Read resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceRequest {
    /// Resource URI
    pub uri: Uri,
}

/// Read resource result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// Resource contents (can be text or binary)
    pub contents: Vec<ResourceContent>,
}

/// Subscribe to resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    /// Resource URI
    pub uri: Uri,
}

/// Unsubscribe from resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeRequest {
    /// Resource URI
    pub uri: Uri,
}

/// Resource updated notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdatedNotification {
    /// Resource URI
    pub uri: Uri,
}

// ============================================================================
// Logging Types
// ============================================================================

/// Log level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Notice level
    Notice,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
    /// Alert level
    Alert,
    /// Emergency level
    Emergency,
}

/// Set log level request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelRequest {
    /// Log level to set
    pub level: LogLevel,
}

/// Set log level result (no data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelResult;

/// Logging notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingNotification {
    /// Log level
    pub level: LogLevel,
    /// Log data
    pub data: serde_json::Value,
    /// Optional logger name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
}

// ============================================================================
// Progress Types
// ============================================================================

/// Progress notification per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    /// The progress token which was given in the initial request
    /// Used to associate this notification with the request that is proceeding
    #[serde(rename = "progressToken")]
    pub progress_token: ProgressToken,
    /// The progress thus far. This should increase every time progress is made,
    /// even if the total is unknown
    pub progress: f64,
    /// Total number of items to process (or total progress required), if known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    /// An optional message describing the current progress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Cancellation notification per MCP 2025-06-18 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledNotification {
    /// The ID of the request to cancel
    /// This MUST correspond to the ID of a request previously issued in the same direction
    #[serde(rename = "requestId")]
    pub request_id: RequestId,
    /// An optional string describing the reason for the cancellation
    /// This MAY be logged or presented to the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ============================================================================
// Sampling Types
// ============================================================================

/// Create message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    /// Messages to include
    pub messages: Vec<SamplingMessage>,
    /// Model preferences
    #[serde(rename = "modelPreferences", skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    /// System prompt
    #[serde(rename = "systemPrompt", skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Include context
    #[serde(rename = "includeContext", skip_serializing_if = "Option::is_none")]
    pub include_context: Option<IncludeContext>,
    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Max tokens
    #[serde(rename = "maxTokens", skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Stop sequences
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Model preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    /// Preferred hints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<ModelHint>>,
    /// Cost priority
    #[serde(rename = "costPriority", skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f64>,
    /// Speed priority
    #[serde(rename = "speedPriority", skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f64>,
    /// Intelligence priority
    #[serde(
        rename = "intelligencePriority",
        skip_serializing_if = "Option::is_none"
    )]
    pub intelligence_priority: Option<f64>,
}

/// Model hint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHint {
    /// Hint name
    pub name: Option<String>,
}

/// Include context options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncludeContext {
    /// No context
    None,
    /// This server only
    ThisServer,
    /// All servers
    AllServers,
}

/// Sampling message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    /// Message role
    pub role: Role,
    /// Message content
    pub content: Content,
}

/// Create message result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageResult {
    /// Role of the created message
    pub role: Role,
    /// Content of the created message
    pub content: Content,
    /// Model used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Stop reason
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

// ============================================================================
// Roots Types
// ============================================================================

/// Filesystem root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    /// Root URI
    pub uri: Uri,
    /// Root name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// List roots request (no parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRootsRequest;

/// List roots result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRootsResult {
    /// Available roots
    pub roots: Vec<Root>,
}

/// Roots list changed notification (no parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsListChangedNotification;

/// Empty result for operations that don't return data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyResult {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let tool = Tool {
            name: "test_tool".to_string(),
            title: Some("Test Tool".to_string()),
            description: Some("A test tool".to_string()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: None,
                additional_properties: None,
            },
            output_schema: None,
            annotations: None,
            meta: None,
        };

        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(tool.name, deserialized.name);
    }

    #[test]
    fn test_content_types() {
        let text_content = ContentBlock::Text(TextContent {
            text: "Hello, World!".to_string(),
            annotations: None,
            meta: None,
        });

        let json = serde_json::to_string(&text_content).unwrap();
        let _deserialized: ContentBlock = serde_json::from_str(&json).unwrap();

        // Test the compatibility alias
        let _compatible: Content = text_content;
    }
}
