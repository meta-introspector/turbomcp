//! Progressive Enhancement Example
//!
//! This example demonstrates TurboMCP's progressive enhancement philosophy:
//! - Start with the simplest transport (STDIO) that works everywhere
//! - Add more advanced transports when needed
//! - Use runtime configuration for deployment flexibility
//!
//! The same server code works across all transports without modification.

use turbomcp::prelude::*;

#[derive(Clone)]
struct FileServer {
    base_path: std::path::PathBuf,
}

#[server]
impl FileServer {
    #[tool("List files in directory")]
    async fn list_files(&self, path: Option<String>) -> McpResult<Vec<String>> {
        let target_path = match path {
            Some(p) => self.base_path.join(p),
            None => self.base_path.clone(),
        };

        if !target_path.exists() || !target_path.is_dir() {
            return Err(mcp_error!("Directory does not exist: {:?}", target_path).into());
        }

        let mut files = Vec::new();
        let entries = std::fs::read_dir(target_path)
            .map_err(|e| mcp_error!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| mcp_error!("Failed to read entry: {}", e))?;
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }

        files.sort();
        Ok(files)
    }

    #[tool("Read file contents")]
    async fn read_file(&self, path: String) -> McpResult<String> {
        let file_path = self.base_path.join(path);

        if !file_path.exists() || !file_path.is_file() {
            return Err(mcp_error!("File does not exist: {:?}", file_path).into());
        }

        let contents = std::fs::read_to_string(file_path)
            .map_err(|e| mcp_error!("Failed to read file: {}", e))?;

        Ok(contents)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = FileServer {
        base_path: std::env::current_dir()?,
    };

    // Progressive enhancement: start simple, add complexity when needed
    match std::env::var("MCP_TRANSPORT")
        .unwrap_or_else(|_| "stdio".to_string())
        .as_str()
    {
        // Level 1: STDIO - works everywhere, no configuration needed
        "stdio" => {
            println!("📝 File server running on STDIO");
            server.run_stdio().await?;
        }

        // Level 2: TCP - network accessible, requires port configuration
        "tcp" => {
            let port = std::env::var("MCP_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse::<u16>()
                .unwrap_or(3000);

            println!("🌐 File server running on TCP port {}", port);
            #[cfg(feature = "tcp")]
            {
                server.run_tcp(format!("0.0.0.0:{}", port)).await?;
            }
            #[cfg(not(feature = "tcp"))]
            {
                eprintln!("TCP transport not available, falling back to STDIO");
                server.run_stdio().await?;
            }
        }

        // Level 3: Unix sockets - IPC, requires path configuration
        "unix" => {
            let socket_path =
                std::env::var("MCP_SOCKET").unwrap_or_else(|_| "/tmp/fileserver.sock".to_string());

            println!("🔌 File server running on Unix socket {}", socket_path);
            #[cfg(all(feature = "unix", unix))]
            {
                let _ = std::fs::remove_file(&socket_path); // Clean up any existing socket
                server.run_unix(socket_path).await?;
            }
            #[cfg(not(all(feature = "unix", unix)))]
            {
                eprintln!("Unix socket transport not available, falling back to STDIO");
                server.run_stdio().await?;
            }
        }

        // Unknown transport: graceful fallback
        transport => {
            eprintln!("Unknown transport '{}', falling back to STDIO", transport);
            server.run_stdio().await?;
        }
    }

    Ok(())
}
