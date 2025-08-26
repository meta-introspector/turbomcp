# TurboMCP CLI

[![Crates.io](https://img.shields.io/crates/v/turbomcp-cli.svg)](https://crates.io/crates/turbomcp-cli)
[![Documentation](https://docs.rs/turbomcp-cli/badge.svg)](https://docs.rs/turbomcp-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Comprehensive command-line tools for developing, testing, debugging, and deploying MCP servers with world-class developer experience.**

## Overview

`turbomcp-cli` provides a complete toolkit for MCP server development. From initial scaffolding to production deployment, the CLI handles server testing, protocol debugging, performance benchmarking, schema validation, and configuration management.

## Key Features

### 🧪 **Server Testing & Validation**
- **Comprehensive testing** - Full MCP protocol compliance validation
- **Tool validation** - Automated testing of tool functionality and schemas
- **Resource testing** - URI template validation and resource access testing  
- **Integration testing** - End-to-end workflow validation
- **Performance testing** - Load testing and benchmarking capabilities

### 🔧 **Protocol Debugging**
- **Message inspection** - Real-time JSON-RPC message logging and analysis
- **Protocol validation** - MCP specification compliance checking
- **Schema debugging** - JSON schema validation and error reporting
- **Transport debugging** - Connection and transport-layer troubleshooting

### 📊 **Performance & Benchmarking**
- **Load testing** - Configurable concurrent request testing
- **Latency analysis** - Request/response timing analysis
- **Throughput measurement** - Messages per second benchmarking
- **Memory profiling** - Resource usage analysis and optimization

### 📋 **Schema Management**
- **Schema export** - Extract JSON schemas from MCP servers
- **Schema validation** - Validate schemas against MCP specification
- **Documentation generation** - Automatic API documentation from schemas
- **Schema comparison** - Diff and compatibility checking between versions

## Installation

### From Crates.io

```bash
# Install latest stable version
cargo install turbomcp-cli

# Install specific version
cargo install turbomcp-cli --version 1.0.0

# Install with all features
cargo install turbomcp-cli --all-features
```

### From Source

```bash
git clone https://github.com/Epistates/turbomcp.git
cd turbomcp
cargo install --path crates/turbomcp-cli
```

### Binary Releases

Download pre-built binaries from [GitHub Releases](https://github.com/Epistates/turbomcp/releases).

## Commands

### Server Testing

#### `tools-list` - List Available Tools

```bash
# List tools via STDIO transport
turbomcp-cli tools-list --transport stdio --command "./my-server"

# List tools via HTTP transport
turbomcp-cli tools-list --transport http --url http://localhost:8080/mcp

# List tools with detailed schemas
turbomcp-cli tools-list --transport stdio --command "./my-server" --detailed

# Output as JSON
turbomcp-cli tools-list --transport stdio --command "./my-server" --output json
```

**Example Output:**
```
Available Tools:
  calculator/add - Add two numbers
    Parameters:
      a: number (required) - First number
      b: number (required) - Second number

  file/read - Read file contents
    Parameters:  
      path: string (required) - File path to read
      encoding: string (optional) - File encoding (default: utf8)

Found 2 tools
```

#### `tools-call` - Execute Tools

```bash
# Call a tool with parameters
turbomcp-cli tools-call \
    --name "calculator/add" \
    --arguments '{"a": 5, "b": 3}' \
    --transport stdio \
    --command "./my-server"

# Call tool with file input
turbomcp-cli tools-call \
    --name "file/process" \
    --arguments @params.json \
    --transport http \
    --url http://localhost:8080/mcp

# Call tool with timeout
turbomcp-cli tools-call \
    --name "long-running-tool" \
    --arguments '{"data": "large-dataset"}' \
    --timeout 60s \
    --transport stdio \
    --command "./my-server"
```

#### `resources-list` - List Available Resources

```bash
# List all resources
turbomcp-cli resources-list --transport stdio --command "./my-server"

# Filter resources by URI pattern
turbomcp-cli resources-list \
    --transport stdio \
    --command "./my-server" \
    --filter "file://*"

# Include resource templates
turbomcp-cli resources-list \
    --transport stdio \
    --command "./my-server" \
    --include-templates
```

#### `resources-read` - Read Resource Content

```bash
# Read a specific resource
turbomcp-cli resources-read \
    --uri "file:///etc/hosts" \
    --transport stdio \
    --command "./my-server"

# Read multiple resources
turbomcp-cli resources-read \
    --uri "file:///var/log/app.log" \
    --uri "file:///etc/config.yaml" \
    --transport stdio \
    --command "./my-server"

# Read with content type detection
turbomcp-cli resources-read \
    --uri "file:///image.png" \
    --binary \
    --transport stdio \
    --command "./my-server"
```

### Server Validation

#### `server-test` - Comprehensive Server Testing

```bash
# Run full server test suite
turbomcp-cli server-test --transport stdio --command "./my-server"

# Test specific categories
turbomcp-cli server-test \
    --transport stdio \
    --command "./my-server" \
    --categories tools,resources,initialization

# Generate test report
turbomcp-cli server-test \
    --transport stdio \
    --command "./my-server" \
    --report test-results.json

# Run with custom test configuration
turbomcp-cli server-test \
    --transport stdio \
    --command "./my-server" \
    --config test-config.toml
```

**Example Test Config (`test-config.toml`):**
```toml
[server]
timeout = "30s"
max_retries = 3

[tools]
# Test all tools automatically
test_all = true
# Custom tool tests
[[tools.custom_tests]]
name = "calculator/add"
parameters = { a = 5, b = 3 }
expected_result = 8

[resources]
test_all = true
# Test specific URI patterns
uri_patterns = ["file://*", "http://*"]

[performance]
concurrent_requests = 10
test_duration = "60s"
```

#### `validate` - Protocol Compliance

```bash
# Validate MCP compliance
turbomcp-cli validate --transport stdio --command "./my-server"

# Validate with specific MCP version
turbomcp-cli validate \
    --transport stdio \
    --command "./my-server" \
    --mcp-version "2025-06-18"

# Validate and fix common issues
turbomcp-cli validate \
    --transport stdio \
    --command "./my-server" \
    --fix-issues
```

### Schema Management

#### `schema-export` - Export Schemas

```bash
# Export all schemas
turbomcp-cli schema-export \
    --transport stdio \
    --command "./my-server" \
    --output schemas/

# Export specific tool schemas
turbomcp-cli schema-export \
    --transport stdio \
    --command "./my-server" \
    --tools calculator/add,file/read \
    --output schemas/tools.json

# Export with documentation
turbomcp-cli schema-export \
    --transport stdio \
    --command "./my-server" \
    --include-docs \
    --format openapi \
    --output api-docs.yaml
```

#### `schema-validate` - Validate Schemas

```bash
# Validate schemas against MCP specification
turbomcp-cli schema-validate \
    --schema schemas/tools.json \
    --spec mcp-2025-06-18

# Validate and show detailed errors
turbomcp-cli schema-validate \
    --schema schemas/ \
    --verbose \
    --show-warnings
```

### Performance & Benchmarking

#### `benchmark` - Performance Testing

```bash
# Basic benchmark
turbomcp-cli benchmark \
    --transport stdio \
    --command "./my-server" \
    --duration 60s

# Concurrent request benchmark
turbomcp-cli benchmark \
    --transport stdio \
    --command "./my-server" \
    --concurrent 10 \
    --requests 1000

# Tool-specific benchmark
turbomcp-cli benchmark \
    --transport stdio \
    --command "./my-server" \
    --tool "calculator/add" \
    --arguments '{"a": 5, "b": 3}' \
    --concurrent 5 \
    --duration 30s
```

**Example Benchmark Output:**
```
TurboMCP Benchmark Results
==========================

Server: ./my-server (stdio)
Duration: 60s
Concurrent Connections: 10

Results:
  Total Requests: 15,429
  Successful: 15,429 (100.0%)
  Failed: 0 (0.0%)
  
  Requests/sec: 257.15 (avg)
  Response Time: 38.9ms (avg)
  
  Percentiles:
    50th: 32ms
    95th: 78ms  
    99th: 156ms
    99.9th: 312ms

Memory Usage:
  Peak RSS: 12.3 MB
  Average: 8.7 MB
```

#### `profile` - Resource Profiling

```bash
# Memory profiling
turbomcp-cli profile memory \
    --transport stdio \
    --command "./my-server" \
    --duration 60s \
    --output memory-profile.json

# CPU profiling
turbomcp-cli profile cpu \
    --transport stdio \
    --command "./my-server" \
    --duration 30s \
    --flamegraph profile.svg
```

### Development Tools

#### `scaffold` - Project Scaffolding

```bash
# Create new MCP server project
turbomcp-cli scaffold new-server \
    --name my-mcp-server \
    --template basic \
    --language rust

# Create advanced server with features
turbomcp-cli scaffold new-server \
    --name enterprise-server \
    --template enterprise \
    --features oauth,metrics,health-checks

# Add tools to existing server
turbomcp-cli scaffold add-tool \
    --name calculator \
    --description "Basic calculator tool" \
    --parameters a:number,b:number
```

#### `dev` - Development Server

```bash
# Run development server with hot reload
turbomcp-cli dev \
    --server ./my-server \
    --watch \
    --reload-on-change

# Development server with debugging
turbomcp-cli dev \
    --server ./my-server \
    --debug \
    --log-level trace \
    --log-format json
```

### Configuration Management

#### `config` - Configuration Management

```bash
# Generate default configuration
turbomcp-cli config init --output turbomcp.toml

# Validate configuration
turbomcp-cli config validate --config turbomcp.toml

# Show effective configuration
turbomcp-cli config show --config turbomcp.toml --resolved
```

**Example Configuration (`turbomcp.toml`):**
```toml
[server]
name = "my-mcp-server"
version = "1.0.0"
transport = "stdio"

[development]
hot_reload = true
debug_logging = true
test_on_change = true

[testing]
timeout = "30s"
max_concurrent = 10
test_categories = ["tools", "resources", "initialization"]

[performance]
benchmark_duration = "60s"
benchmark_concurrent = 5
memory_limit = "100MB"

[deployment]
health_checks = true
metrics_enabled = true
graceful_shutdown = true
```

## Transport Support

### STDIO Transport

```bash
# Direct command execution
turbomcp-cli tools-list --transport stdio --command "./server"

# With arguments
turbomcp-cli tools-list --transport stdio --command "python3" --args "-m,my_server"

# With working directory
turbomcp-cli tools-list \
    --transport stdio \
    --command "./server" \
    --working-dir "/path/to/server"
```

### HTTP Transport

```bash
# Basic HTTP connection
turbomcp-cli tools-list --transport http --url "http://localhost:8080/mcp"

# With authentication
turbomcp-cli tools-list \
    --transport http \
    --url "https://api.example.com/mcp" \
    --header "Authorization: Bearer $TOKEN"

# With custom headers
turbomcp-cli tools-list \
    --transport http \
    --url "http://localhost:8080/mcp" \
    --header "X-API-Version: v1" \
    --header "X-Client: turbomcp-cli"
```

### WebSocket Transport

```bash
# WebSocket connection
turbomcp-cli tools-list --transport websocket --url "ws://localhost:8080/mcp"

# Secure WebSocket
turbomcp-cli tools-list \
    --transport websocket \
    --url "wss://api.example.com/mcp" \
    --header "Authorization: Bearer $TOKEN"
```

### TCP Transport

```bash
# TCP connection
turbomcp-cli tools-list --transport tcp --address "localhost:8080"

# With connection timeout
turbomcp-cli tools-list \
    --transport tcp \
    --address "localhost:8080" \
    --timeout 30s
```

### Unix Socket Transport

```bash
# Unix socket connection
turbomcp-cli tools-list --transport unix --socket "/tmp/mcp.sock"

# With permissions
turbomcp-cli tools-list \
    --transport unix \
    --socket "/tmp/mcp.sock" \
    --permissions 660
```

## Output Formats

### JSON Output

```bash
# JSON output for tools
turbomcp-cli tools-list \
    --transport stdio \
    --command "./server" \
    --output json | jq '.tools[0].name'

# Pretty-printed JSON
turbomcp-cli tools-list \
    --transport stdio \
    --command "./server" \
    --output json-pretty
```

### Table Output

```bash
# Table format (default)
turbomcp-cli tools-list --transport stdio --command "./server" --output table

# Custom table columns
turbomcp-cli tools-list \
    --transport stdio \
    --command "./server" \
    --output table \
    --columns name,description,parameters
```

### CSV Output

```bash
# CSV export
turbomcp-cli tools-list \
    --transport stdio \
    --command "./server" \
    --output csv > tools.csv

# Custom CSV delimiter
turbomcp-cli tools-list \
    --transport stdio \
    --command "./server" \
    --output csv \
    --delimiter ";" > tools.csv
```

## Advanced Usage

### Batch Operations

```bash
# Run multiple commands from file
turbomcp-cli batch --commands commands.txt

# Example commands.txt:
# tools-list --transport stdio --command "./server1"
# tools-list --transport stdio --command "./server2" 
# benchmark --transport stdio --command "./server1" --duration 30s
```

### Pipeline Integration

```bash
# Use with CI/CD pipelines
turbomcp-cli validate \
    --transport stdio \
    --command "./server" \
    --exit-code \
    --output junit > test-results.xml

# Health check integration
turbomcp-cli health-check \
    --transport http \
    --url "http://localhost:8080/mcp" \
    --timeout 10s \
    --exit-code
```

### Custom Scripts

```bash
# Run custom test scripts
turbomcp-cli script run test-suite.js \
    --transport stdio \
    --command "./server"

# JavaScript test script example (test-suite.js):
// Test server initialization
const server = await connect();
await server.initialize();

// Test tools
const tools = await server.listTools();
assert(tools.length > 0, "Server should have tools");

// Test specific tool
const result = await server.callTool("calculator/add", {a: 5, b: 3});
assert(result === 8, "Calculator should work");
```

## Configuration Files

### Global Configuration

```bash
# Location: ~/.config/turbomcp/config.toml
[defaults]
transport = "stdio"
timeout = "30s"
output = "table"

[stdio]
default_args = []
default_working_dir = "."

[http]
default_headers = ["User-Agent: turbomcp-cli/1.0"]
verify_ssl = true

[testing]
concurrent_requests = 5
test_timeout = "60s"
```

### Project Configuration

```bash
# Location: ./turbomcp.toml (project root)
[server]
command = "./target/debug/my-server"
transport = "stdio"

[development]
watch_files = ["src/**/*.rs", "Cargo.toml"]
test_on_change = true

[testing]
test_categories = ["initialization", "tools", "resources"]
custom_tests = "tests/integration.toml"
```

## Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/mcp-test.yml
name: MCP Server Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install TurboMCP CLI
        run: cargo install turbomcp-cli
        
      - name: Build server
        run: cargo build --release
        
      - name: Test MCP server
        run: |
          turbomcp-cli validate \
            --transport stdio \
            --command "./target/release/my-server" \
            --exit-code
            
      - name: Benchmark server
        run: |
          turbomcp-cli benchmark \
            --transport stdio \
            --command "./target/release/my-server" \
            --duration 30s \
            --report benchmark.json
            
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: |
            benchmark.json
```

### Docker Integration

```dockerfile
FROM rust:1.89 as builder

# Install TurboMCP CLI
RUN cargo install turbomcp-cli

# Build server
COPY . /app
WORKDIR /app
RUN cargo build --release

# Test server during build
RUN turbomcp-cli validate \
    --transport stdio \
    --command "./target/release/server" \
    --exit-code

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/server /usr/local/bin/
COPY --from=builder /usr/local/cargo/bin/turbomcp-cli /usr/local/bin/

# Health check using CLI
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD turbomcp-cli health-check \
    --transport stdio \
    --command "/usr/local/bin/server" \
    --timeout 5s \
    --exit-code

ENTRYPOINT ["/usr/local/bin/server"]
```

## Development

### Building from Source

```bash
git clone https://github.com/Epistates/turbomcp.git
cd turbomcp/crates/turbomcp-cli
cargo build --release
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Test with different transports
cargo test --features all-transports
```

## Related Tools

- **[turbomcp](../turbomcp/)** - Main TurboMCP framework
- **[turbomcp-server](../turbomcp-server/)** - Server implementation  
- **[turbomcp-client](../turbomcp-client/)** - Client implementation
- **[turbomcp-transport](../turbomcp-transport/)** - Transport protocols

## External Resources

- **[MCP Specification](https://modelcontextprotocol.io/)** - Official protocol specification
- **[Claude Desktop](https://claude.ai/desktop)** - AI assistant with MCP support
- **[JSON-RPC 2.0](https://www.jsonrpc.org/specification)** - Underlying RPC protocol

## License

Licensed under the [MIT License](../../LICENSE).

---

*Part of the [TurboMCP](../../) high-performance Rust SDK for the Model Context Protocol.*