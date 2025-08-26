//! HTTP SSE server example (no macros)

use std::sync::Arc;
use turbomcp_framework::session::{SessionConfig, SessionManager};
use turbomcp_framework::sse_server::{start_sse_server, SseServerConfig};
use turbomcp_server::ServerError;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    tracing_subscriber::fmt::init();

    let session_mgr = Arc::new(SessionManager::new(SessionConfig::default()));
    let cfg = SseServerConfig::default();
    start_sse_server(cfg, session_mgr, None)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))
}
