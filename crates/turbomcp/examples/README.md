# TurboMCP Examples

A comprehensive, progressive series of examples that teach TurboMCP from beginner to ready-to-use implementations. Each example is self-contained, thoroughly documented, and designed to build upon previous concepts.

## 🎯 Progressive Learning Path

Our examples are designed as a **complete learning journey** from simple concepts to production deployment:

| Example | Time | Level | Description |
|---------|------|-------|-------------|
| [`01_hello_world.rs`](01_hello_world.rs) | 5 min | Beginner | Your first MCP server with one tool |
| [`02_tools_basics.rs`](02_tools_basics.rs) | 10 min | Beginner | Essential tool patterns and error handling |
| [`03_macros_vs_builders.rs`](03_macros_vs_builders.rs) | 15 min | Intermediate | Compare macro vs builder APIs side-by-side |
| [`04_comprehensive_server.rs`](04_comprehensive_server.rs) | 20 min | Intermediate | Full MCP server with tools, resources, and prompts |
| [`07_performance.rs`](07_performance.rs) | 25 min | Advanced | High-throughput optimization and memory management |
| [`09_oauth_authentication.rs`](09_oauth_authentication.rs) | 30 min | Expert | OAuth 2.0 authentication with Google/GitHub providers |
| [`10_http_server.rs`](10_http_server.rs) | 25 min | Advanced | HTTP server with REST API, SSE, and WebSocket support |
| [`11_child_process.rs`](11_child_process.rs) | 20 min | Advanced | Child process management for multi-language MCP servers |
| [`transport_showcase.rs`](transport_showcase.rs) | 15 min | Intermediate | Runtime transport selection (STDIO, HTTP, TCP, Unix) |
| [`graceful_shutdown.rs`](graceful_shutdown.rs) | 15 min | Advanced | Graceful shutdown patterns for production deployment |

## 🚀 Quick Start

```bash
# Start your TurboMCP journey with the simplest example
cargo run --example 01_hello_world

# Or jump to a specific concept you need
cargo run --example 06_macro_showcase

```

## 🧠 What You'll Learn

### Beginner Level (Examples 01-02)
- **Basic MCP Concepts**: Understanding servers, tools, and the MCP protocol
- **Server Setup**: Creating your first working MCP server
- **Tool Development**: Writing functions that AI assistants can call
- **Error Handling**: Proper validation and error responses
- **Parameter Types**: Simple strings, numbers, booleans, and optional parameters

### Intermediate Level (Examples 03-04)  
- **API Comparison**: When to use macros vs builders
- **Full Server Features**: Tools, resources, prompts, and state management
- **Structured Parameters**: Complex JSON objects with validation
- **Resource Handling**: URI templates and dynamic resource access
- **Context Usage**: Logging, tracing, and request correlation

### Advanced Level (Examples 05-06)
- **Production Patterns**: Circuit breakers, retry logic, timeouts
- **Operation Cancellation**: Graceful handling of interrupted requests
- **Complete Macro Usage**: Every TurboMCP macro with real examples
- **Comprehensive Validation**: Input sanitization and security
- **Advanced Async**: Concurrent operations and resource management

### Expert Level (Examples 07-08)
- **Performance Optimization**: Memory pools, connection pooling, caching
- **High Throughput**: Request batching and parallel processing
- **Production Deployment**: Configuration, monitoring, health checks
- **Integration Patterns**: Database connections, external services, observability

## 📚 Detailed Example Breakdown

### 01_hello_world.rs - Your First MCP Server
```rust
#[turbomcp::server(name = "HelloWorld", version = "1.0.0")]
impl HelloWorldServer {
    #[tool("Say hello to someone")]
    async fn hello(&self, name: Option<String>) -> McpResult<String> {
        let who = name.unwrap_or_else(|| "World".to_string());
        Ok(format!("Hello, {}! Welcome to TurboMCP! 🦀⚡", who))
    }
}
```
**Learn**: Basic server structure, simple tools, optional parameters

### 02_tools_basics.rs - Essential Tool Patterns  
```rust
#[tool("Add two numbers together")]
async fn add(&self, a: f64, b: f64) -> McpResult<f64> {
    if a.is_infinite() || b.is_infinite() {
        return Err(McpError::invalid_request("Cannot add infinite numbers"));
    }
    Ok(a + b)
}
```
**Learn**: Input validation, error handling, mathematical operations

### 03_macros_vs_builders.rs - API Comparison
Shows the same functionality implemented with both approaches:
- **Macro API**: Clean, declarative, automatic validation
- **Builder API**: Full control, manual handling, dynamic registration  
**Learn**: When to choose each approach, trade-offs, best practices

### 04_comprehensive_server.rs - Full MCP Features
```rust
#[tool("Process file securely")]
#[resource("file://secure/{path}")]  
#[prompt("Generate documentation for {project}")]
```
**Learn**: All MCP capabilities, security constraints, real-world patterns

### 05_advanced_patterns.rs - Production Patterns
```rust
async fn with_circuit_breaker<F, T>(&self, operation: F) -> McpResult<T> 
where F: Future<Output = McpResult<T>> {
    // Circuit breaker implementation
}
```
**Learn**: Reliability patterns, fault tolerance, graceful degradation

### 06_macro_showcase.rs - Complete Macro Reference
Demonstrates **every** TurboMCP macro:
- `#[server]`, `#[tool]`, `#[resource]`, `#[prompt]` 
- `mcp_text!()`, `mcp_error!()`, `tool_result!()`
**Learn**: Complete macro ecosystem, advanced usage patterns

### 07_performance.rs - High-Throughput Optimization
```rust
async fn batch_process(&self, operations: Vec<Operation>) -> McpResult<Vec<Result>> {
    // Connection pooling, memory management, parallel processing
}
```
**Learn**: Performance optimization, memory management, scalability

### 09_oauth_authentication.rs - OAuth 2.0 Authentication
```rust
#[server(
    name = "AuthenticatedMCPServer",
    version = "1.0.0",
    description = "MCP server with world-class OAuth 2.0 authentication"
)]
impl AuthenticatedServer {
    #[tool("Get authenticated user profile")]
    async fn get_user_profile(&self) -> McpResult<String> {
        // Access authenticated user information
    }

    #[tool("Start OAuth authentication flow")]
    async fn start_oauth_flow(&self, provider: String) -> McpResult<String> {
        // Initiate OAuth flow with Google, GitHub, or Microsoft
    }
}
```
**Learn**: OAuth 2.0 authentication, provider setup, PKCE security, session management

### graceful_shutdown.rs - Production Deployment
```rust
async fn graceful_shutdown_handler() -> Result<(), Box<dyn std::error::Error>> {
    // Graceful shutdown with signal handling
    let (server, shutdown_handle) = server.into_server_with_shutdown()?;
}
```
**Learn**: Production deployment, graceful shutdown, signal handling

## 🎯 Choose Your Learning Path

### 🚀 Complete Journey (2+ hours)
Work through all examples in order for comprehensive mastery:
```bash
cargo run --example 01_hello_world
cargo run --example 02_tools_basics
cargo run --example 03_macros_vs_builders
cargo run --example 04_comprehensive_server
cargo run --example 07_performance
cargo run --example 09_oauth_authentication
cargo run --example graceful_shutdown
```

### ⚡ Quick Start (30 minutes)
Essential examples for immediate productivity:
```bash
cargo run --example 01_hello_world      # 5 min - Basic concepts
cargo run --example 02_tools_basics     # 10 min - Tool patterns  
cargo run --example 04_comprehensive_server # 15 min - Complete reference
```

### 🔐 Authentication Path (45 minutes)
For applications requiring OAuth 2.0 authentication:
```bash
cargo run --example 01_hello_world      # 5 min - Basic concepts
cargo run --example 04_comprehensive_server # 15 min - Full server features
cargo run --example 09_oauth_authentication # 25 min - OAuth 2.0 integration
```

### 🏗️ Builder API Path (45 minutes)
If you prefer explicit control over magic:
```bash
cargo run --example 01_hello_world      # 5 min - See the macro way
cargo run --example 03_macros_vs_builders # 15 min - Compare approaches
cargo run --example 05_advanced_patterns # 25 min - Production patterns
```

### ⚙️ Production Path (1+ hour)
Ready to deploy? Focus on operational excellence:
```bash
cargo run --example 05_advanced_patterns # 25 min - Reliability patterns
cargo run --example 07_performance       # 25 min - High throughput
cargo run --example 08_integration       # 30 min - Production deployment
```

## 🔧 TurboMCP API Reference

### Attribute Macros - Progressive Enhancement Design

TurboMCP macros follow the **"Simple things are simple, complex things are possible"** philosophy:

#### `#[server]` - MCP Server Definition
Transform structs into MCP servers with automatic trait implementation:
```rust
#[server]                                    // Simple: automatic name/version  
#[server(name = "MyServer")]                 // Basic: custom name
#[server(                                    // Advanced: full configuration
    name = "ProductionServer",
    version = "2.0.0", 
    description = "Enterprise MCP server"
)]
```

#### `#[tool]` - Tool Functions
Mark methods as MCP tools with automatic parameter parsing:
```rust
#[tool("Description")]                       // Simple: description only
#[tool("Advanced calculator")]               // Most common usage
```

#### `#[resource]` - Resource Handlers (Progressive Enhancement)
Create resource handlers with world-class flexibility:
```rust
#[resource]                                  // Simple: function name as resource
#[resource("file://{path}")]                 // Common: URI template override
#[resource(                                  // Advanced: comprehensive configuration
    uri = "secure://data/{id}", 
    name = "secure_data",
    tags = ["security", "enterprise"]
)]
```

#### `#[prompt]` - Prompt Generators (Progressive Enhancement) 
Build prompt generators with maximum utility:
```rust
#[prompt("Generate code review")]            // Simple: description only
#[prompt(                                    // Advanced: full metadata
    desc = "Generate comprehensive code review",
    name = "code_reviewer", 
    tags = ["code", "review", "analysis"]
)]
```

### Helper Macros  
- **`mcp_text!()`** - Create ContentBlock structures (rare - for manual CallToolResult building)  
- **`mcp_error!()`** - Create structured ServerError types for error handling
- **`tool_result!()`** - Create CallToolResult manually (rare - usually auto-handled by #[tool])

### Macro Usage Guidelines

**`format!()` - Use in 90% of cases:**
- ✅ Tool function return values: `Ok(format!("Result: {}", value))`
- ✅ Logging: `ctx.info(&format!("Processing: {}", item))`  
- ✅ Error messages: `Err(McpError::invalid_request(&format!("Bad input: {}", input)))`

**`mcp_text!()` - Rare advanced usage:**
- ⚠️ Manual CallToolResult construction (usually unnecessary)
- ⚠️ Building complex ContentBlock structures  
- ⚠️ When bypassing the automatic conversion from #[tool] macros

**`mcp_error!()` - For structured error types:**
- ✅ Creating ServerError types: `mcp_error!("Connection failed: {}", error)`
- ✅ Use with `Err(mcp_error!(...))` for proper error conversion

**`tool_result!()` - Very rare:**
- ⚠️ Manual CallToolResult creation (automatic via #[tool] macro)
- ⚠️ Complex multi-content responses

### Common Patterns

**Error Handling**
```rust
#[tool("Safe operation")]
async fn safe_op(&self, input: String) -> McpResult<String> {
    if input.is_empty() {
        return Err(McpError::invalid_request("Input cannot be empty"));
    }
    Ok(input.to_uppercase())
}
```

**Context Logging**
```rust  
#[tool("Logged operation")]
async fn logged_op(&self, ctx: Context, data: String) -> McpResult<String> {
    ctx.info(&format!("Processing {} bytes", data.len())).await?;
    let result = process_data(data);
    ctx.info("Processing completed successfully").await?;
    Ok(result)
}
```

**Structured Parameters**
```rust
#[derive(Deserialize, Serialize)]
struct UserRequest {
    name: String,
    email: String,
    active: bool,
}

#[tool("Create user with validation")]
async fn create_user(&self, req: UserRequest) -> McpResult<String> {
    // Automatic JSON parsing and validation
    Ok(format!("Created user: {}", req.name))
}
```

## 🚀 Next Steps

1. **Start with example 01** - Get your first server running in 5 minutes
2. **Progress through the series** - Each example builds on the previous ones
3. **Copy examples as templates** - Use them as starting points for your projects
4. **Explore the codebase** - Check `crates/*/src/` for implementation details
5. **Read the documentation** - Visit [docs.rs/turbomcp](https://docs.rs/turbomcp) for API reference

## 📝 Best Practices

### Development
- **Follow the progression** - Don't skip ahead without understanding basics
- **Use the macro API** - It's more ergonomic and less error-prone than builders
- **Handle errors gracefully** - Always validate inputs and provide helpful error messages
- **Add comprehensive logging** - Use `Context` for operation tracing and debugging
- **Test extensively** - All examples can be run locally with `cargo run --example <name>`

### Production
- **Review security examples** - See `04_comprehensive_server.rs` for security patterns
- **Implement reliability patterns** - Use circuit breakers and retries from `05_advanced_patterns.rs`  
- **Optimize for performance** - Apply techniques from `07_performance.rs`
- **Deploy with monitoring** - Follow patterns in `08_integration.rs`

## 🆘 Troubleshooting

**Examples won't compile?**
```bash
# Make sure you're in the project root
cd /path/to/turbomcp
cargo check --example 01_hello_world
```

**Runtime errors?**
```bash
# Enable debug logging
RUST_LOG=debug cargo run --example 01_hello_world
```

**Need help with MCP clients?**
- Try [Claude Desktop](https://claude.ai/download) for testing your servers
- Use the MCP Inspector for debugging protocol interactions
- Check [MCP Specification](https://modelcontextprotocol.io/) for protocol details

**Still stuck?**
- Open an issue on [GitHub](https://github.com/Epistates/turbomcp/issues)
- Join discussions at [GitHub Discussions](https://github.com/Epistates/turbomcp/discussions)
- Check existing issues for common solutions

## 🌟 Contributing

Found ways to improve these examples? We welcome contributions!

1. Fork the repository
2. Create a feature branch (`git checkout -b improve-examples`)
3. Make your improvements
4. Add tests if applicable  
5. Submit a pull request

**Example improvement ideas:**
- Additional error handling patterns
- More real-world use cases
- Performance optimizations
- Security enhancements
- Better documentation

---

**Happy coding with TurboMCP!** 🦀⚡

*These examples represent years of production experience distilled into practical, copy-and-paste solutions. Use them as stepping stones to build amazing MCP servers.*