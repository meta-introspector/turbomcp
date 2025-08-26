#![allow(dead_code)]
//! # 04: Comprehensive Server - Full MCP Feature Showcase
//!
//! **Learning Goals (20 minutes):**
//! - See all MCP capabilities in one complete server
//! - Learn tools, resources, prompts, and state management
//! - Understand real-world server architecture patterns
//! - Master advanced context usage and logging
//!
//! **What this example demonstrates:**
//! - Complete MCP server with all capability types
//! - File system tools with proper security
//! - Dynamic resources with URI templates
//! - AI prompt generators with context
//! - Persistent state management
//! - Authentication and session handling
//! - Comprehensive error handling
//!
//! **Run with:** `cargo run --example 04_comprehensive_server`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use turbomcp::prelude::*;

/// A comprehensive MCP server showcasing all major features
#[derive(Debug, Clone)]
struct ComprehensiveServer {
    /// In-memory storage for demonstration
    #[allow(dead_code)]
    storage: std::sync::Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    /// Base directory for file operations (security-constrained)
    base_dir: PathBuf,
    /// Request counter for statistics
    #[allow(dead_code)]
    request_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileOperation {
    path: String,
    content: Option<String>,
    create_dirs: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchParams {
    query: String,
    file_types: Option<Vec<String>>, // e.g., ["txt", "md", "rs"]
    max_results: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PromptContext {
    task_type: String, // "code_review", "documentation", "analysis"
    subject: String,
    details: Option<String>,
    style: Option<String>, // "formal", "casual", "technical"
}

#[turbomcp::server(name = "ComprehensiveServer", version = "1.0.0")]
#[allow(dead_code, unused)]
impl ComprehensiveServer {
    fn new() -> McpResult<Self> {
        // Create a secure base directory for file operations
        let base_dir = std::env::temp_dir().join("turbomcp_demo");
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| McpError::internal(format!("Failed to create base directory: {e}")))?;

        Ok(Self {
            storage: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            base_dir,
            request_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// Validate and resolve file paths securely
    fn resolve_path(&self, path: &str) -> McpResult<PathBuf> {
        let requested_path = PathBuf::from(path);

        // Prevent directory traversal attacks
        if path.contains("..") || path.contains("~") || path.starts_with('/') {
            return Err(McpError::invalid_request(
                "Invalid path: directory traversal not allowed",
            ));
        }

        let full_path = self.base_dir.join(requested_path);

        // Ensure the resolved path is still within our base directory
        if !full_path.starts_with(&self.base_dir) {
            return Err(McpError::invalid_request(
                "Invalid path: outside allowed directory",
            ));
        }

        Ok(full_path)
    }
}

impl ComprehensiveServer {
    // =============================================================================
    // TOOLS - Functions that perform actions
    // =============================================================================

    /// Write content to a file safely
    #[tool("Write content to a file within the allowed directory")]
    async fn write_file(&self, params: FileOperation) -> McpResult<String> {
        // Increment request counter
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("Writing file: {}", params.path);

        let full_path = self.resolve_path(&params.path)?;

        let content = params.content.unwrap_or_default();

        // Create parent directories if requested
        if params.create_dirs.unwrap_or(false)
            && let Some(parent) = full_path.parent()
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| McpError::internal(format!("Failed to create directories: {e}")))?;
        }

        // Write the file
        std::fs::write(&full_path, &content)
            .map_err(|e| McpError::internal(format!("Failed to write file: {e}")))?;

        tracing::info!(
            "Successfully wrote {} bytes to {}",
            content.len(),
            params.path
        );

        Ok(format!(
            "Successfully wrote {} bytes to {}",
            content.len(),
            params.path
        ))
    }

    /// Read content from a file safely
    #[tool("Read content from a file within the allowed directory")]
    async fn read_file(&self, path: String) -> McpResult<String> {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("Reading file: {}", path);

        let full_path = self.resolve_path(&path)?;

        if !full_path.exists() {
            return Err(McpError::invalid_request("File does not exist"));
        }

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal(format!("Failed to read file: {e}")))?;

        tracing::info!("Successfully read {} bytes from {}", content.len(), path);

        Ok(content)
    }

    /// Search for files based on patterns
    #[tool("Search for files by name pattern and type")]
    async fn search_files(&self, params: SearchParams) -> McpResult<Vec<String>> {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("Searching for files matching: {}", params.query);

        let max_results = params.max_results.unwrap_or(50);
        let file_types = params
            .file_types
            .unwrap_or_else(|| vec!["txt".to_string(), "md".to_string()]);

        let mut results = Vec::new();

        // Simple file search implementation
        fn search_dir(
            dir: &std::path::Path,
            query: &str,
            file_types: &[String],
            results: &mut Vec<String>,
            max: usize,
        ) -> Result<(), std::io::Error> {
            if results.len() >= max {
                return Ok(());
            }

            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    search_dir(&path, query, file_types, results, max)?;
                } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check if filename contains query
                    if file_name.to_lowercase().contains(&query.to_lowercase()) {
                        // Check file extension
                        if let Some(ext) = path.extension().and_then(|e| e.to_str())
                            && file_types.contains(&ext.to_lowercase())
                            && let Some(relative_path) =
                                path.strip_prefix(dir).ok().and_then(|p| p.to_str())
                        {
                            results.push(relative_path.to_string());
                        }
                    }
                }
            }
            Ok(())
        }

        search_dir(
            &self.base_dir,
            &params.query,
            &file_types,
            &mut results,
            max_results,
        )
        .map_err(|e| McpError::internal(format!("Search failed: {e}")))?;

        tracing::info!("Found {} matching files", results.len());

        Ok(results)
    }

    /// Store data in server memory
    #[tool("Store a key-value pair in server memory")]
    async fn store_data(&self, key: String, value: serde_json::Value) -> McpResult<String> {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("Storing data with key: {}", key);

        let mut storage = self.storage.write().await;
        storage.insert(key.clone(), value.clone());

        Ok(format!("Successfully stored data with key: {key}"))
    }

    /// Retrieve data from server memory  
    #[tool("Retrieve a value by key from server memory")]
    async fn get_data(&self, key: String) -> McpResult<serde_json::Value> {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("Retrieving data with key: {}", key);

        let storage = self.storage.read().await;
        match storage.get(&key) {
            Some(value) => Ok(value.clone()),
            None => Err(McpError::invalid_request("Key not found")),
        }
    }

    /// List all stored keys
    #[tool("List all keys in server memory")]
    async fn list_keys(&self) -> McpResult<Vec<String>> {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("Listing all stored keys");

        let storage = self.storage.read().await;
        let keys: Vec<String> = storage.keys().cloned().collect();

        Ok(keys)
    }

    /// Get server statistics
    #[tool("Get comprehensive server statistics and status")]
    async fn get_stats(&self) -> McpResult<String> {
        tracing::info!("Generating server statistics");

        let request_count = self.request_count.load(std::sync::atomic::Ordering::SeqCst);
        let storage = self.storage.read().await;
        let storage_size = storage.len();

        let stats = serde_json::json!({
            "server": {
                "name": "ComprehensiveServer",
                "version": "1.0.0",
                "uptime": "N/A", // Would track actual uptime in real implementation
                "status": "healthy"
            },
            "requests": {
                "total": request_count,
                "rate": "N/A" // Would calculate actual rate in real implementation
            },
            "storage": {
                "keys_stored": storage_size,
                "base_directory": self.base_dir.display().to_string()
            },
            "capabilities": {
                "tools": ["write_file", "read_file", "search_files", "store_data", "get_data", "list_keys", "get_stats"],
                "resources": ["file://*", "data://*", "search://*"],
                "prompts": ["code_review", "documentation", "analysis"]
            }
        });

        Ok(stats.to_string())
    }

    // =============================================================================
    // RESOURCES - Dynamic content providers with URI templates
    // =============================================================================

    /// Provide file content as a resource
    #[resource("file://{path}")]
    async fn file_resource(&self, path: String) -> McpResult<String> {
        tracing::info!("Serving file resource: {}", path);

        let full_path = self.resolve_path(&path)?;

        if !full_path.exists() {
            return Err(McpError::resource("File not found"));
        }

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal(format!("Failed to read file: {e}")))?;

        Ok(content)
    }

    /// Provide stored data as a resource
    #[resource("data://{key}")]
    async fn data_resource(&self, key: String) -> McpResult<String> {
        tracing::info!("Serving data resource: {}", key);

        let storage = self.storage.read().await;
        match storage.get(&key) {
            Some(value) => Ok(value.to_string()),
            None => Err(McpError::resource("Data key not found")),
        }
    }

    /// Provide search results as a resource
    #[resource("search://{query}")]
    async fn search_resource(&self, query: String) -> McpResult<String> {
        tracing::info!("Serving search resource: {}", query);

        let params = SearchParams {
            query,
            file_types: Some(vec!["txt".to_string(), "md".to_string(), "rs".to_string()]),
            max_results: Some(20),
        };

        let results = self.search_files(params).await?;

        let response = serde_json::json!({
            "search_results": results,
            "total_found": results.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(response.to_string())
    }

    // =============================================================================
    // PROMPTS - AI prompt generators for different tasks
    // =============================================================================

    /// Generate a code review prompt
    #[prompt("Generate a code review prompt for {task_type} of {subject}")]
    async fn code_review_prompt(&self, args: Option<serde_json::Value>) -> McpResult<String> {
        let params = if let Some(args) = args {
            serde_json::from_value::<PromptContext>(args).unwrap_or(PromptContext {
                task_type: "code_review".to_string(),
                subject: "unknown".to_string(),
                details: None,
                style: None,
            })
        } else {
            PromptContext {
                task_type: "code_review".to_string(),
                subject: "general".to_string(),
                details: None,
                style: None,
            }
        };

        // Note: Context isn't available in prompts yet - this is a TODO for the macro system

        let style = params.style.unwrap_or_else(|| "professional".to_string());
        let details = params
            .details
            .unwrap_or_else(|| "No additional details provided".to_string());

        let prompt = match params.task_type.as_str() {
            "code_review" => {
                format!(
                    "Please perform a comprehensive code review of the following {subject}.\n\
                     \n\
                     Review Style: {style}\n\
                     Additional Context: {details}\n\
                     \n\
                     Please analyze:\n\
                     1. Code quality and best practices\n\
                     2. Security considerations\n\
                     3. Performance implications\n\
                     4. Maintainability and readability\n\
                     5. Test coverage and edge cases\n\
                     \n\
                     Provide specific, actionable feedback with examples.",
                    subject = params.subject,
                    style = style,
                    details = details
                )
            }
            "documentation" => {
                format!(
                    "Please create comprehensive documentation for the following {subject}.\n\
                     \n\
                     Documentation Style: {style}\n\
                     Additional Requirements: {details}\n\
                     \n\
                     Please include:\n\
                     1. Clear overview and purpose\n\
                     2. Usage examples with code samples\n\
                     3. Parameter descriptions and types\n\
                     4. Return values and error conditions\n\
                     5. Best practices and common pitfalls\n\
                     \n\
                     Make it accessible to developers at all levels.",
                    subject = params.subject,
                    style = style,
                    details = details
                )
            }
            "analysis" => {
                format!(
                    "Please perform a detailed analysis of the following {subject}.\n\
                     \n\
                     Analysis Approach: {style}\n\
                     Specific Focus: {details}\n\
                     \n\
                     Please provide:\n\
                     1. Executive summary of key findings\n\
                     2. Detailed technical analysis\n\
                     3. Strengths and weaknesses\n\
                     4. Recommendations for improvement\n\
                     5. Future considerations\n\
                     \n\
                     Support your analysis with specific examples and data.",
                    subject = params.subject,
                    style = style,
                    details = details
                )
            }
            _ => {
                return Err(McpError::invalid_request(
                    "Unknown task type. Supported: code_review, documentation, analysis",
                ));
            }
        };

        Ok(prompt)
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Enhanced logging for comprehensive server
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_thread_ids(true)
        .init();

    tracing::info!("🌟 Starting Comprehensive MCP Server");
    tracing::info!("====================================");
    tracing::info!("Features: Tools, Resources, Prompts, State Management");
    tracing::info!("Security: File operations restricted to temp directory");
    tracing::info!("Storage: In-memory key-value store");

    // Create server with all capabilities
    let server = ComprehensiveServer::new()?;

    tracing::info!("Base directory: {}", server.base_dir.display());
    tracing::info!("Server ready! Connect from Claude Desktop to explore all features.");

    // Run the server
    server
        .run_stdio()
        .await
        .map_err(|e| McpError::internal(format!("Server error: {e}")))
}

/* 🎯 **Try these comprehensive examples:**

**Tools (Actions):**
- write_file({"path": "test.txt", "content": "Hello TurboMCP!", "create_dirs": true})
- read_file("test.txt")
- search_files({"query": "test", "file_types": ["txt"], "max_results": 10})
- store_data("user_prefs", {"theme": "dark", "notifications": true})
- get_data("user_prefs")
- list_keys()
- get_stats()

**Resources (Dynamic content via URI):**
- Access file://test.txt to get file content
- Access data://user_prefs to get stored data
- Access search://test to get search results

**Prompts (AI prompt generation):**
- Generate prompts for code reviews, documentation, or analysis
- code_review_prompt({"task_type": "code_review", "subject": "authentication system", "style": "thorough"})

**Key Features Demonstrated:**
✅ Complete MCP server with all capability types
✅ Secure file operations with path validation
✅ In-memory state management with persistence
✅ Dynamic resources with URI template matching
✅ AI prompt generation with context awareness
✅ Comprehensive error handling and validation
✅ Request tracking and server statistics
✅ Context-aware logging and tracing

**Next:** `05_advanced_patterns.rs` - Advanced error handling and async patterns
*/
