#![allow(dead_code)]
//! # 06: Macro Showcase - Complete TurboMCP Macro Reference
//!
//! **Learning Goals (20 minutes):**
//! - See ALL TurboMCP attribute macros in action
//! - Learn helper macro usage patterns  
//! - Understand macro composition and integration
//! - Master the complete TurboMCP macro ecosystem
//!
//! **What this example demonstrates:**
//! - `#[server]` - Server definition and automatic trait implementation
//! - `#[tool]` - Tool handlers with automatic schema generation
//! - `#[prompt]` - Prompt handlers for AI assistants
//! - `#[resource]` - Resource handlers with URI templates
//! - `mcp_text!()` - Ergonomic text content creation
//! - `mcp_error!()` - Structured error creation
//! - `tool_result!()` - Tool result wrapper creation
//!
//! **Run with:** `cargo run --example 06_macro_showcase`

use serde::{Deserialize, Serialize};
use turbomcp::prelude::*;
use turbomcp::{prompt, resource, server, tool};

/// Comprehensive development environment server showcasing ALL TurboMCP macros
///
/// This server demonstrates a complete development environment with:
/// - Code generation tools
/// - Project management prompts  
/// - Configuration resources
/// - File system operations
#[derive(Debug, Clone)]
struct DevEnvironmentServer {
    /// Current project directory
    project_dir: std::path::PathBuf,
    /// Available programming languages
    languages: Vec<String>,
    /// Server configuration
    config: ServerConfig,
}

#[derive(Debug, Clone)]
struct ServerConfig {
    max_file_size: usize,
    allowed_extensions: Vec<String>,
    template_dir: std::path::PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct CodeGenerationRequest {
    language: String,
    template_type: String,
    project_name: String,
    features: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalysisResult {
    score: u32,
    issues: Vec<String>,
    suggestions: Vec<String>,
    metrics: CodeMetrics,
}

#[derive(Debug, Deserialize, Serialize)]
struct CodeMetrics {
    lines_of_code: u32,
    complexity_score: f64,
    maintainability_index: f64,
}

#[server]
impl DevEnvironmentServer {
    /// Create a new development environment server
    fn new() -> Self {
        Self {
            project_dir: std::env::current_dir().unwrap_or_else(|_| "/tmp".into()),
            languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
            ],
            config: ServerConfig {
                max_file_size: 10 * 1024 * 1024, // 10MB
                allowed_extensions: vec![
                    ".rs".to_string(),
                    ".py".to_string(),
                    ".js".to_string(),
                    ".ts".to_string(),
                    ".go".to_string(),
                ],
                template_dir: "/templates".into(),
            },
        }
    }

    /// Generate a new project using templates
    #[tool("Generate a new project from templates with specified features")]
    async fn generate_project(&self, params: CodeGenerationRequest) -> McpResult<String> {
        // Log project generation start
        tracing::info!("Starting project generation: {}", params.project_name);

        // Validate language is supported
        if !self.languages.contains(&params.language) {
            return Err(mcp_error!(
                "Unsupported language: {}. Supported: {}",
                params.language,
                self.languages.join(", ")
            )
            .into());
        }

        // Simulate project generation
        tracing::info!("Creating directory structure...");
        tracing::info!("Copying template files...");
        tracing::info!("Configuring build system...");

        let features_info = if params.features.is_empty() {
            "basic template".to_string()
        } else {
            format!("features: {}", params.features.join(", "))
        };

        tracing::info!("Project generation completed successfully");
        Ok(format!(
            "✅ Successfully generated {} project '{}' with {}",
            params.language, params.project_name, features_info
        ))
    }

    /// Analyze code quality and provide suggestions
    #[tool("Analyze code quality and provide improvement suggestions")]
    async fn analyze_code(&self, file_path: String, detailed: Option<bool>) -> McpResult<String> {
        let detailed = detailed.unwrap_or(false);

        tracing::info!("Analyzing code at: {}", file_path);

        // Validate file path and extension
        let path = std::path::Path::new(&file_path);
        if !path.exists() {
            return Err(mcp_error!("File not found: {}", file_path).into());
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| format!(".{s}"));

        if let Some(ext) = &extension
            && !self.config.allowed_extensions.contains(ext)
        {
            return Err(mcp_error!(
                "Unsupported file type: {}. Supported: {}",
                ext,
                self.config.allowed_extensions.join(", ")
            )
            .into());
        }

        tracing::info!("Running static analysis...");
        tracing::info!("Calculating complexity metrics...");
        tracing::info!("Checking code style...");

        let analysis = AnalysisResult {
            score: 85,
            issues: vec![
                "Consider extracting long method".to_string(),
                "Add documentation comments".to_string(),
            ],
            suggestions: vec![
                "Use more descriptive variable names".to_string(),
                "Consider breaking up complex functions".to_string(),
            ],
            metrics: CodeMetrics {
                lines_of_code: 247,
                complexity_score: 6.8,
                maintainability_index: 73.2,
            },
        };

        if detailed {
            Ok(format!(
                "📊 Detailed Analysis of {}\n\
                Score: {}/100\n\
                Lines of Code: {}\n\
                Complexity: {:.1}\n\
                Maintainability: {:.1}\n\n\
                Issues Found:\n{}\n\n\
                Suggestions:\n{}",
                file_path,
                analysis.score,
                analysis.metrics.lines_of_code,
                analysis.metrics.complexity_score,
                analysis.metrics.maintainability_index,
                analysis
                    .issues
                    .iter()
                    .map(|i| format!("• {i}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                analysis
                    .suggestions
                    .iter()
                    .map(|s| format!("• {s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        } else {
            Ok(format!(
                "📊 Analysis complete: {}/100 score, {} issues, {} suggestions",
                analysis.score,
                analysis.issues.len(),
                analysis.suggestions.len()
            ))
        }
    }

    /// Run automated tests for a project
    #[tool("Run automated tests and report results")]
    async fn run_tests(
        &self,
        project_path: String,
        test_type: Option<String>,
    ) -> McpResult<String> {
        let test_type = test_type.unwrap_or_else(|| "unit".to_string());

        tracing::info!("Running {} tests in: {}", test_type, project_path);

        // Validate project directory
        let path = std::path::Path::new(&project_path);
        if !path.exists() || !path.is_dir() {
            return Err(mcp_error!("Invalid project directory: {}", project_path).into());
        }

        tracing::info!("Discovering test files...");
        tracing::info!("Executing test suite...");

        // Simulate test execution
        let passed = 23;
        let failed = 2;
        let skipped = 1;

        if failed > 0 {
            tracing::warn!(
                "Some tests failed: {} passed, {} failed, {} skipped",
                passed,
                failed,
                skipped
            );
        } else {
            tracing::info!("All tests passed!");
        }

        Ok(format!(
            "🧪 Test Results: {passed} passed, {failed} failed, {skipped} skipped"
        ))
    }

    /// Generate coding prompts for AI assistants
    #[prompt("Generate comprehensive coding prompts for development tasks")]
    async fn coding_prompt(&self, args: Option<serde_json::Value>) -> McpResult<String> {
        let task_type = args
            .as_ref()
            .and_then(|v| v.get("task"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let language = args
            .as_ref()
            .and_then(|v| v.get("language"))
            .and_then(|v| v.as_str())
            .unwrap_or("rust");

        tracing::info!("Generating {} coding prompt for {}", task_type, language);

        let prompt = match task_type {
            "refactor" => format!(
                "You are an expert {language} developer. Please help refactor the following code:\n\n\
                Goals:\n\
                - Improve code readability and maintainability\n\
                - Follow {language} best practices and idioms\n\
                - Optimize performance where possible\n\
                - Add comprehensive documentation\n\n\
                Please provide:\n\
                1. Refactored code with explanations\n\
                2. Summary of changes made\n\
                3. Performance implications"
            ),
            "debug" => format!(
                "You are a debugging expert for {language}. Help identify and fix issues in the code:\n\n\
                Analysis needed:\n\
                - Identify potential bugs and logic errors\n\
                - Check for edge cases and error handling\n\
                - Validate {language} best practices\n\
                - Suggest testing strategies\n\n\
                Please provide:\n\
                1. Issue analysis\n\
                2. Fixed code with explanations\n\
                3. Prevention strategies"
            ),
            "optimize" => format!(
                "You are a performance optimization expert for {language}. Analyze and optimize:\n\n\
                Optimization targets:\n\
                - Runtime performance improvements\n\
                - Memory usage optimization\n\
                - Algorithm efficiency\n\
                - {language} specific optimizations\n\n\
                Please provide:\n\
                1. Performance analysis\n\
                2. Optimized implementation\n\
                3. Benchmarking suggestions"
            ),
            _ => format!(
                "You are an expert {language} developer. Analyze the following code and provide:\n\n\
                1. Code review with detailed feedback\n\
                2. Suggestions for improvements\n\
                3. Best practice recommendations\n\
                4. Testing strategies\n\
                5. Documentation improvements\n\n\
                Focus on {language} idioms and conventions."
            ),
        };

        Ok(prompt)
    }

    /// Access project configuration files
    #[resource("config://project/{section}")]
    async fn project_config(&self, _uri: String, section: String) -> McpResult<String> {
        tracing::info!("Accessing config section: {}", section);

        let config_content = match section.as_str() {
            "build" => serde_json::json!({
                "target": "release",
                "optimization": "speed",
                "features": ["default"],
                "dependencies": {
                    "core": "1.0.0",
                    "utils": "0.9.0"
                }
            }),
            "test" => serde_json::json!({
                "runner": "default",
                "timeout": 300,
                "parallel": true,
                "coverage": true
            }),
            "deployment" => serde_json::json!({
                "environment": "production",
                "replicas": 3,
                "resources": {
                    "cpu": "2",
                    "memory": "4Gi"
                }
            }),
            _ => {
                return Err(mcp_error!(
                    "Unknown config section: {}. Available: build, test, deployment",
                    section
                )
                .into());
            }
        };

        Ok(serde_json::to_string_pretty(&config_content)
            .unwrap_or_else(|_| "Failed to serialize config".to_string()))
    }

    /// Access development templates
    #[resource("template://{language}/{template_type}")]
    async fn dev_template(
        &self,
        _uri: String,
        language: String,
        template_type: String,
    ) -> McpResult<String> {
        tracing::info!("Fetching {} template for {}", template_type, language);

        if !self.languages.contains(&language) {
            return Err(mcp_error!(
                "Unsupported language: {}. Supported: {}",
                language,
                self.languages.join(", ")
            )
            .into());
        }

        let template = match (language.as_str(), template_type.as_str()) {
            ("rust", "basic") => templates::RUST_BASIC,
            ("rust", "cli") => templates::RUST_CLI,
            ("python", "basic") => templates::PYTHON_BASIC,
            ("javascript", "basic") => templates::JS_BASIC,
            _ => {
                return Err(mcp_error!(
                    "Template not found: {} for {}. Check available templates.",
                    template_type,
                    language
                )
                .into());
            }
        };

        Ok(template.to_string())
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    tracing::info!("🚀 Starting TurboMCP Macro Showcase Server");
    tracing::info!("This example demonstrates ALL TurboMCP macros");

    // Create and run the server
    let server = DevEnvironmentServer::new();
    server.run_stdio().await?;

    Ok(())
}

// Template content - in a real implementation, these would be separate files
mod templates {
    pub const RUST_BASIC: &str = r#"
fn main() {
    println!("Hello, world!");
}
"#;

    pub const RUST_CLI: &str = r#"
use clap::Parser;

#[derive(Parser)]
#[command(name = "my-tool")]
struct Args {
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    println!("CLI tool starting...");
}
"#;

    pub const PYTHON_BASIC: &str = r#"
def main():
    print("Hello, world!")

if __name__ == "__main__":
    main()
"#;

    pub const JS_BASIC: &str = r#"
function main() {
    console.log("Hello, world!");
}

main();
"#;
}

/* 🎯 **Try it out:**

   1. **Basic Usage:**
   ```bash
   cargo run --example 06_macro_showcase
   ```

   2. **Test the tools via JSON-RPC:**
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "tools/call",
     "params": {
       "name": "generate_project",
       "arguments": {
         "language": "rust",
         "template_type": "cli",
         "project_name": "my-awesome-tool",
         "features": ["logging", "config"]
       }
     }
   }
   ```

   3. **Test prompts:**
   ```json
   {
     "jsonrpc": "2.0",
     "id": 2,
     "method": "prompts/get",
     "params": {
       "name": "coding_prompt",
       "arguments": {
         "task": "refactor",
         "language": "rust"
       }
     }
   }
   ```
*/

/* 📝 **Macro Summary:**

**Server Macro:**
- `#[server]` - Transforms struct into MCP server with automatic trait implementations

**Tool Macros:**
- `#[tool("description")]` - Creates MCP tool handlers with automatic schema generation
- Parameters are automatically extracted from function signatures
- Return types are automatically converted to MCP responses

**Prompt Macros:**
- `#[prompt("description")]` - Creates AI assistant prompt generators
- Flexible argument handling via `serde_json::Value`

**Resource Macros:**
- `#[resource("uri://template/{param}")]` - URI template-based resource handlers
- Automatic parameter extraction from URI paths

**Helper Macros:**
- `mcp_text!("format", args...)` - Ergonomic text content creation
- `mcp_error!("format", args...)` - Structured error creation
- `tool_result!()` - Tool result wrapper creation

**Next Example:** `07_performance_optimization.rs` - High-performance server patterns
*/
