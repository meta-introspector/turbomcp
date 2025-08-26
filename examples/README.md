# TurboMCP Examples

This directory contains comprehensive examples to help you get started with TurboMCP, from basic usage to advanced patterns.

## 🚀 Quick Start

The easiest way to understand TurboMCP is to run an example:

```bash
# Run the comprehensive macro demo
cargo run --example comprehensive_macros

# Or start with a simple demo
cargo run --example turbomcp_demo
```

## 📚 Examples Overview

### Beginner Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`minimal_turbomcp.rs`](minimal_turbomcp.rs) | Absolute minimal server | Bare minimum setup |
| [`basic.rs`](basic.rs) | Basic server template | Configuration, logging |
| [`turbomcp_demo.rs`](turbomcp_demo.rs) | Simple tools and context | `#[tool]` macro, Context usage |

### Macro Examples

| Example | Description | Macros Demonstrated |
|---------|-------------|-------------------|
| [`comprehensive_macros.rs`](comprehensive_macros.rs) | **All macros showcase** | `#[server]`, `#[tool]`, `#[resource]`, `#[prompt]`, helpers |

### Advanced Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`hello_world.rs`](hello_world.rs) | Low-level ServerBuilder API | Manual server construction |
| [`with_state.rs`](with_state.rs) | Stateful server operations | State management, persistence |
| [`http_server.rs`](http_server.rs) | HTTP/SSE deployment | Web server, session management |
| [`custom_tools.rs`](custom_tools.rs) | Advanced tool patterns | Structured params, validation |

### Specialized Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`simd_performance.rs`](simd_performance.rs) | High-performance processing | SIMD acceleration, benchmarks |
| [`security_headers_demo.rs`](security_headers_demo.rs) | Security best practices | Authentication, headers |
| [`tools_and_schema.rs`](tools_and_schema.rs) | Schema generation | Type safety, validation |

## 🎯 Learning Path

### 1. Start Here (5 minutes)
```bash
# Run the minimal example to see a working server
cargo run --example minimal_turbomcp
```

### 2. Learn the Basics (10 minutes)
```bash
# Understand the high-level macro API  
cargo run --example turbomcp_demo
```

### 3. See All Features (15 minutes)
```bash
# Comprehensive example with all macros
cargo run --example comprehensive_macros
```

### 4. Explore Advanced Patterns (30+ minutes)
- [`custom_tools.rs`](custom_tools.rs) - Advanced tool implementations
- [`http_server.rs`](http_server.rs) - Web deployment
- [`simd_performance.rs`](simd_performance.rs) - Performance optimization

## 🏗️ Architecture Overview

TurboMCP provides two main APIs:

### High-Level Macro API (Recommended)
```rust
use turbomcp::prelude::*;

#[turbomcp::server]
struct MyServer { /* ... */ }

impl MyServer {
    #[tool("Description")]
    async fn my_tool(&self, param: String) -> McpResult<String> {
        Ok(param)
    }
}
```

### Low-Level Builder API
```rust
use turbomcp_server::ServerBuilder;

let server = ServerBuilder::new()
    .name("MyServer")
    .tool("my_tool", /* handler */)
    .build();
```

## 🔧 Macro Reference

### `#[server]` - Server Definition
```rust
#[server(name = "MyServer", version = "1.0.0")]
struct MyServer {
    // Your server fields
}
```

### `#[tool]` - Tool Handlers
```rust
#[tool("Tool description for AI")]
async fn my_tool(&self, param: Type) -> McpResult<ReturnType> {
    // Tool implementation
}
```

### `#[resource]` - Resource Providers
```rust
#[resource("protocol://path/{param}")]
async fn my_resource(&self, param: String) -> McpResult<String> {
    // Resource implementation
}
```

### `#[prompt]` - Prompt Generators
```rust
#[prompt("Generate {type} for {purpose}")]
async fn my_prompt(&self, type_: String, purpose: String) -> McpResult<String> {
    // Prompt generation logic
}
```

### Helper Macros
```rust
// Structured content
let content = mcp_text!("Hello, {}!", name);

// Error creation
let error = mcp_error!("Something went wrong: {}", details);

// Tool results
let result = tool_result!(content);
```

## 🔍 Common Patterns

### Error Handling
```rust
#[tool("Safe division")]
async fn divide(&self, a: f64, b: f64) -> McpResult<f64> {
    if b == 0.0 {
        Err(McpError::InvalidRequest("Division by zero".into()))
    } else {
        Ok(a / b)
    }
}
```

### Input Validation
```rust
#[tool("Process text")]
async fn process(&self, text: String) -> McpResult<String> {
    if text.is_empty() {
        return Err(McpError::InvalidRequest("Text cannot be empty".into()));
    }
    if text.len() > 1000 {
        return Err(McpError::InvalidRequest("Text too long".into()));
    }
    Ok(text.to_uppercase())
}
```

### Context Usage
```rust
#[tool("Log and process")]
async fn process(&self, ctx: Context, data: String) -> McpResult<String> {
    ctx.info(&format!("Processing {} bytes", data.len())).await?;
    
    // Your processing logic
    let result = data.to_uppercase();
    
    ctx.info("Processing completed").await?;
    Ok(result)
}
```

### Structured Parameters
```rust
#[derive(Deserialize)]
struct Params {
    name: String,
    age: u32,
    active: bool,
}

#[tool("Process user data")]
async fn process_user(&self, params: Params) -> McpResult<String> {
    Ok(format!("User {} is {} years old and is {}", 
        params.name, 
        params.age, 
        if params.active { "active" } else { "inactive" }
    ))
}
```

## 🚀 Next Steps

1. **Run the examples** that match your use case
2. **Copy and modify** examples as starting points
3. **Read the documentation** at [docs.rs/turbomcp](https://docs.rs/turbomcp)
4. **Check out the tests** in `crates/*/tests/` for more patterns
5. **Join the community** on [GitHub Discussions](https://github.com/Epistates/turbomcp/discussions)

## 📝 Tips for Success

- **Start simple** - Begin with `minimal_turbomcp.rs` or `turbomcp_demo.rs`
- **Use the macros** - The high-level API is much more ergonomic
- **Handle errors properly** - Always use `McpResult<T>` and proper error types
- **Add logging** - Use the `Context` parameter for operation logging
- **Validate inputs** - Check parameters before processing
- **Test locally** - All examples can be run with `cargo run --example <name>`

## 🆘 Need Help?

- **Examples not working?** Make sure you're in the project root directory
- **Compilation errors?** Check that you have the latest Rust version
- **Runtime issues?** Enable logging with `RUST_LOG=debug cargo run --example <name>`
- **Questions?** Open an issue or discussion on GitHub

Happy coding with TurboMCP! 🦀⚡