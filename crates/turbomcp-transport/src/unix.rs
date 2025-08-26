//! Unix domain socket transport implementation for MCP

use async_trait::async_trait;
use bytes::BytesMut;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::core::{
    Transport, TransportCapabilities, TransportError, TransportMessage, TransportMetrics,
    TransportResult, TransportState, TransportType,
};
use turbomcp_core::MessageId;

/// Unix domain socket transport implementation
#[derive(Debug)]
pub struct UnixTransport {
    /// Socket path
    socket_path: PathBuf,
    /// Server mode flag
    is_server: bool,
    /// Message sender
    sender: Option<mpsc::UnboundedSender<TransportMessage>>,
    /// Message receiver
    receiver: Option<mpsc::UnboundedReceiver<TransportMessage>>,
    /// Transport capabilities
    capabilities: TransportCapabilities,
    /// Current state
    state: TransportState,
    /// Transport metrics
    metrics: TransportMetrics,
}

impl UnixTransport {
    /// Create a new Unix socket transport for server mode
    #[must_use]
    pub fn new_server(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            is_server: true,
            sender: None,
            receiver: None,
            capabilities: TransportCapabilities {
                supports_bidirectional: true,
                supports_streaming: true,
                max_message_size: Some(64 * 1024 * 1024), // 64MB
                ..Default::default()
            },
            state: TransportState::Disconnected,
            metrics: TransportMetrics::default(),
        }
    }

    /// Create a new Unix socket transport for client mode
    #[must_use]
    pub fn new_client(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            is_server: false,
            sender: None,
            receiver: None,
            capabilities: TransportCapabilities {
                supports_bidirectional: true,
                supports_streaming: true,
                max_message_size: Some(64 * 1024 * 1024), // 64MB
                ..Default::default()
            },
            state: TransportState::Disconnected,
            metrics: TransportMetrics::default(),
        }
    }

    /// Start Unix socket server
    async fn start_server(&mut self) -> TransportResult<()> {
        // Remove existing socket file if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).map_err(|e| {
                TransportError::ConfigurationError(format!(
                    "Failed to remove existing socket file: {e}"
                ))
            })?;
        }

        info!("Starting Unix socket server at {:?}", self.socket_path);
        self.state = TransportState::Connecting;

        let listener = UnixListener::bind(&self.socket_path).map_err(|e| {
            self.state = TransportState::Failed {
                reason: format!("Failed to bind: {e}"),
            };
            TransportError::ConnectionFailed(format!("Failed to bind Unix socket listener: {e}"))
        })?;

        let (tx, rx) = mpsc::unbounded_channel();
        self.sender = Some(tx.clone());
        self.receiver = Some(rx);
        self.state = TransportState::Connected;

        // Accept connections in background
        let socket_path = self.socket_path.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        info!("Accepted Unix socket connection");
                        let sender = tx.clone();
                        let path = socket_path.clone();
                        // Handle connection in separate task
                        tokio::spawn(async move {
                            if let Err(e) = handle_unix_connection(stream, sender, path).await {
                                error!("Unix socket connection handler failed: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept Unix socket connection: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect to Unix socket server
    async fn connect_client(&mut self) -> TransportResult<()> {
        info!("Connecting to Unix socket at {:?}", self.socket_path);
        self.state = TransportState::Connecting;

        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            self.state = TransportState::Failed {
                reason: format!("Failed to connect: {e}"),
            };
            TransportError::ConnectionFailed(format!("Failed to connect to Unix socket: {e}"))
        })?;

        let (tx, rx) = mpsc::unbounded_channel();
        self.sender = Some(tx.clone());
        self.receiver = Some(rx);
        self.state = TransportState::Connected;

        // Handle connection
        let socket_path = self.socket_path.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_unix_connection(stream, tx, socket_path).await {
                error!("Unix socket client connection handler failed: {}", e);
            }
        });

        Ok(())
    }
}

/// Handle a Unix socket connection with proper message framing
async fn handle_unix_connection(
    stream: UnixStream,
    message_sender: mpsc::UnboundedSender<TransportMessage>,
    socket_path: PathBuf,
) -> TransportResult<()> {
    debug!("Handling Unix socket connection for {:?}", socket_path);

    let (read_half, _write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let mut buffer = BytesMut::with_capacity(8192);

    loop {
        // Read message length prefix (4 bytes, big-endian)
        let mut length_bytes = [0u8; 4];
        match reader.read_exact(&mut length_bytes).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("Unix socket connection closed by peer: {:?}", socket_path);
                break;
            }
            Err(e) => {
                error!("Failed to read message length: {}", e);
                return Err(TransportError::ReceiveFailed(format!(
                    "Read length error: {e}"
                )));
            }
        }

        let message_length = u32::from_be_bytes(length_bytes) as usize;

        // Validate message size
        if message_length > 64 * 1024 * 1024 {
            // 64MB limit
            error!(
                "Message too large: {} bytes from {:?}",
                message_length, socket_path
            );
            return Err(TransportError::ProtocolError("Message too large".into()));
        }

        if message_length == 0 {
            warn!("Received zero-length message from {:?}", socket_path);
            continue;
        }

        // Read message payload
        buffer.clear();
        buffer.resize(message_length, 0);

        match reader.read_exact(&mut buffer).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to read message payload: {}", e);
                return Err(TransportError::ReceiveFailed(format!(
                    "Read payload error: {e}"
                )));
            }
        }

        // Parse message to validate JSON format
        match serde_json::from_slice::<serde_json::Value>(&buffer) {
            Ok(value) => {
                let id = value
                    .get("id")
                    .cloned()
                    .unwrap_or_else(|| serde_json::Value::String(uuid::Uuid::new_v4().to_string()));
                let message_id = match id {
                    serde_json::Value::String(s) => MessageId::from(s),
                    serde_json::Value::Number(n) => MessageId::from(n.as_i64().unwrap_or_default()),
                    _ => MessageId::from(uuid::Uuid::new_v4()),
                };
                let transport_msg = TransportMessage::new(message_id, buffer.clone().freeze());

                if message_sender.send(transport_msg).is_err() {
                    warn!(
                        "Message receiver dropped, closing connection for {:?}",
                        socket_path
                    );
                    break;
                }
            }
            Err(e) => {
                error!("Failed to parse message from {:?}: {}", socket_path, e);
                // Skip invalid messages but keep connection open
            }
        }
    }

    debug!(
        "Unix socket connection handler finished for {:?}",
        socket_path
    );
    Ok(())
}

#[async_trait]
impl Transport for UnixTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Unix
    }

    fn capabilities(&self) -> &TransportCapabilities {
        &self.capabilities
    }

    async fn state(&self) -> TransportState {
        self.state.clone()
    }

    async fn connect(&mut self) -> TransportResult<()> {
        if self.is_server {
            self.start_server().await
        } else {
            self.connect_client().await
        }
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        info!("Stopping Unix socket transport");
        self.state = TransportState::Disconnecting;
        self.sender = None;
        self.receiver = None;

        // Clean up socket file if we're the server
        if self.is_server
            && self.socket_path.exists()
            && let Err(e) = std::fs::remove_file(&self.socket_path)
        {
            debug!("Failed to remove socket file: {}", e);
        }

        self.state = TransportState::Disconnected;
        Ok(())
    }

    async fn send(&mut self, message: TransportMessage) -> TransportResult<()> {
        if let Some(ref sender) = self.sender {
            self.metrics.messages_sent += 1;
            self.metrics.bytes_sent += message.size() as u64;

            sender.send(message).map_err(|e| {
                TransportError::SendFailed(format!("Failed to send message via Unix socket: {e}"))
            })?;
            Ok(())
        } else {
            Err(TransportError::ConnectionFailed(
                "Unix socket transport not connected".into(),
            ))
        }
    }

    async fn receive(&mut self) -> TransportResult<Option<TransportMessage>> {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.try_recv() {
                Ok(message) => {
                    self.metrics.messages_received += 1;
                    self.metrics.bytes_received += message.size() as u64;
                    Ok(Some(message))
                }
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    self.state = TransportState::Failed {
                        reason: "Channel disconnected".into(),
                    };
                    Err(TransportError::ReceiveFailed(
                        "Unix socket transport channel closed".into(),
                    ))
                }
            }
        } else {
            Err(TransportError::ConnectionFailed(
                "Unix socket transport not connected".into(),
            ))
        }
    }

    async fn metrics(&self) -> TransportMetrics {
        self.metrics.clone()
    }

    fn endpoint(&self) -> Option<String> {
        Some(format!("unix://{}", self.socket_path.display()))
    }
}

/// Unix socket transport configuration
#[derive(Debug, Clone)]
pub struct UnixConfig {
    /// Socket file path
    pub socket_path: PathBuf,
    /// File permissions for the socket
    pub permissions: Option<u32>,
    /// Buffer size
    pub buffer_size: usize,
    /// Cleanup socket file on disconnect
    pub cleanup_on_disconnect: bool,
}

impl Default for UnixConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/tmp/turbomcp.sock"),
            permissions: Some(0o600), // Owner read/write only
            buffer_size: 8192,
            cleanup_on_disconnect: true,
        }
    }
}

/// Unix socket transport builder
#[derive(Debug)]
pub struct UnixTransportBuilder {
    config: UnixConfig,
    is_server: bool,
}

impl UnixTransportBuilder {
    /// Create a new Unix socket transport builder for server mode
    #[must_use]
    pub fn new_server() -> Self {
        Self {
            config: UnixConfig::default(),
            is_server: true,
        }
    }

    /// Create a new Unix socket transport builder for client mode
    #[must_use]
    pub fn new_client() -> Self {
        Self {
            config: UnixConfig::default(),
            is_server: false,
        }
    }

    /// Set socket path
    pub fn socket_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.socket_path = path.into();
        self
    }

    /// Set file permissions
    #[must_use]
    pub const fn permissions(mut self, permissions: u32) -> Self {
        self.config.permissions = Some(permissions);
        self
    }

    /// Set buffer size
    #[must_use]
    pub const fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Enable or disable socket cleanup on disconnect
    #[must_use]
    pub const fn cleanup_on_disconnect(mut self, enabled: bool) -> Self {
        self.config.cleanup_on_disconnect = enabled;
        self
    }

    /// Build the Unix socket transport
    #[must_use]
    pub fn build(self) -> UnixTransport {
        if self.is_server {
            UnixTransport::new_server(self.config.socket_path)
        } else {
            UnixTransport::new_client(self.config.socket_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_unix_config_default() {
        let config = UnixConfig::default();
        assert_eq!(config.socket_path, Path::new("/tmp/turbomcp.sock"));
        assert_eq!(config.permissions, Some(0o600));
        assert_eq!(config.buffer_size, 8192);
        assert!(config.cleanup_on_disconnect);
    }

    #[test]
    fn test_unix_transport_builder_server() {
        let transport = UnixTransportBuilder::new_server()
            .socket_path("/tmp/test-server.sock")
            .permissions(0o644)
            .buffer_size(4096)
            .build();

        assert_eq!(transport.socket_path, Path::new("/tmp/test-server.sock"));
        assert!(transport.is_server);
        assert!(matches!(transport.state, TransportState::Disconnected));
    }

    #[test]
    fn test_unix_transport_builder_client() {
        let transport = UnixTransportBuilder::new_client()
            .socket_path("/tmp/test-client.sock")
            .build();

        assert_eq!(transport.socket_path, Path::new("/tmp/test-client.sock"));
        assert!(!transport.is_server);
    }

    #[tokio::test]
    async fn test_unix_transport_state() {
        let transport = UnixTransportBuilder::new_server().build();

        assert_eq!(transport.state().await, TransportState::Disconnected);
        assert_eq!(transport.transport_type(), TransportType::Unix);
    }

    #[test]
    fn test_unix_transport_endpoint() {
        let path = PathBuf::from("/tmp/test.sock");
        let transport = UnixTransport::new_server(path.clone());

        assert_eq!(
            transport.endpoint(),
            Some(format!("unix://{}", path.display()))
        );
    }

    #[test]
    fn test_unix_config_builder_pattern() {
        let config = UnixConfig {
            socket_path: PathBuf::from("/tmp/custom.sock"),
            permissions: Some(0o755),
            buffer_size: 16384,
            cleanup_on_disconnect: false,
        };

        assert_eq!(config.socket_path, Path::new("/tmp/custom.sock"));
        assert_eq!(config.permissions, Some(0o755));
        assert_eq!(config.buffer_size, 16384);
        assert!(!config.cleanup_on_disconnect);
    }
}
