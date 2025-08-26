//! TCP transport implementation for MCP

use async_trait::async_trait;
use bytes::BytesMut;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::core::{
    Transport, TransportCapabilities, TransportError, TransportMessage, TransportMetrics,
    TransportResult, TransportState, TransportType,
};
use turbomcp_core::MessageId;

/// TCP transport implementation
#[derive(Debug)]
pub struct TcpTransport {
    /// Local address to bind to
    bind_addr: SocketAddr,
    /// Remote address to connect to (for client mode)
    remote_addr: Option<SocketAddr>,
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

impl TcpTransport {
    /// Create a new TCP transport for server mode
    #[must_use]
    pub fn new_server(bind_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            remote_addr: None,
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

    /// Create a new TCP transport for client mode
    #[must_use]
    pub fn new_client(bind_addr: SocketAddr, remote_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            remote_addr: Some(remote_addr),
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

    /// Start TCP server
    async fn start_server(&mut self) -> TransportResult<()> {
        info!("Starting TCP server on {}", self.bind_addr);
        self.state = TransportState::Connecting;

        let listener = TcpListener::bind(self.bind_addr).await.map_err(|e| {
            self.state = TransportState::Failed {
                reason: format!("Failed to bind TCP listener: {e}"),
            };
            TransportError::ConnectionFailed(format!("Failed to bind TCP listener: {e}"))
        })?;

        let (tx, rx) = mpsc::unbounded_channel();
        self.sender = Some(tx.clone());
        self.receiver = Some(rx);
        self.state = TransportState::Connected;

        // Accept connections in background
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("Accepted TCP connection from {}", addr);
                        let sender = tx.clone();
                        // Handle connection in separate task
                        tokio::spawn(async move {
                            if let Err(e) = handle_tcp_connection(stream, addr, sender).await {
                                error!("TCP connection handler failed for {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept TCP connection: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect to TCP server
    async fn connect_client(&mut self) -> TransportResult<()> {
        let remote_addr = self.remote_addr.ok_or_else(|| {
            TransportError::ConfigurationError("No remote address set for client".into())
        })?;

        info!("Connecting to TCP server at {}", remote_addr);
        self.state = TransportState::Connecting;

        let stream = TcpStream::connect(remote_addr).await.map_err(|e| {
            self.state = TransportState::Failed {
                reason: format!("Failed to connect: {e}"),
            };
            TransportError::ConnectionFailed(format!("Failed to connect to TCP server: {e}"))
        })?;

        let (tx, rx) = mpsc::unbounded_channel();
        self.sender = Some(tx.clone());
        self.receiver = Some(rx);
        self.state = TransportState::Connected;

        // Handle connection
        tokio::spawn(async move {
            if let Err(e) = handle_tcp_connection(stream, remote_addr, tx).await {
                error!("TCP client connection handler failed: {}", e);
            }
        });

        Ok(())
    }
}

/// Handle a TCP connection with proper message framing
async fn handle_tcp_connection(
    stream: TcpStream,
    addr: SocketAddr,
    message_sender: mpsc::UnboundedSender<TransportMessage>,
) -> TransportResult<()> {
    debug!("Handling TCP connection from {}", addr);

    let (read_half, _write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let mut buffer = BytesMut::with_capacity(8192);

    loop {
        // Read message length prefix (4 bytes, big-endian)
        let mut length_bytes = [0u8; 4];
        match reader.read_exact(&mut length_bytes).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("TCP connection closed by peer: {}", addr);
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
            error!("Message too large: {} bytes from {}", message_length, addr);
            return Err(TransportError::ProtocolError("Message too large".into()));
        }

        if message_length == 0 {
            warn!("Received zero-length message from {}", addr);
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
                    warn!("Message receiver dropped, closing connection to {}", addr);
                    break;
                }
            }
            Err(e) => {
                error!("Failed to parse message from {}: {}", addr, e);
                // Skip invalid messages but keep connection open
            }
        }
    }

    debug!("TCP connection handler finished for {}", addr);
    Ok(())
}

#[async_trait]
impl Transport for TcpTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }

    fn capabilities(&self) -> &TransportCapabilities {
        &self.capabilities
    }

    async fn state(&self) -> TransportState {
        self.state.clone()
    }

    async fn connect(&mut self) -> TransportResult<()> {
        if self.remote_addr.is_some() {
            // Client mode
            self.connect_client().await
        } else {
            // Server mode
            self.start_server().await
        }
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        info!("Stopping TCP transport");
        self.state = TransportState::Disconnecting;
        self.sender = None;
        self.receiver = None;
        self.state = TransportState::Disconnected;
        Ok(())
    }

    async fn send(&mut self, message: TransportMessage) -> TransportResult<()> {
        if let Some(ref sender) = self.sender {
            self.metrics.messages_sent += 1;
            self.metrics.bytes_sent += message.size() as u64;

            sender.send(message).map_err(|e| {
                TransportError::SendFailed(format!("Failed to send message via TCP: {e}"))
            })?;
            Ok(())
        } else {
            Err(TransportError::ConnectionFailed(
                "TCP transport not connected".into(),
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
                        "TCP transport channel closed".into(),
                    ))
                }
            }
        } else {
            Err(TransportError::ConnectionFailed(
                "TCP transport not connected".into(),
            ))
        }
    }

    async fn metrics(&self) -> TransportMetrics {
        self.metrics.clone()
    }

    fn endpoint(&self) -> Option<String> {
        if let Some(remote) = self.remote_addr {
            Some(format!("tcp://{remote}"))
        } else {
            Some(format!("tcp://{}", self.bind_addr))
        }
    }
}

/// TCP transport configuration
#[derive(Debug, Clone)]
pub struct TcpConfig {
    /// Bind address for server mode
    pub bind_addr: SocketAddr,
    /// Remote address for client mode
    pub remote_addr: Option<SocketAddr>,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Keep-alive settings
    pub keep_alive: bool,
    /// Buffer sizes
    pub buffer_size: usize,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080"
                .parse()
                .expect("Default TCP bind address should be valid"),
            remote_addr: None,
            connect_timeout_ms: 5000,
            keep_alive: true,
            buffer_size: 8192,
        }
    }
}

/// TCP transport builder
#[derive(Debug)]
pub struct TcpTransportBuilder {
    config: TcpConfig,
}

impl TcpTransportBuilder {
    /// Create a new TCP transport builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: TcpConfig::default(),
        }
    }

    /// Set bind address
    #[must_use]
    pub const fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.config.bind_addr = addr;
        self
    }

    /// Set remote address for client mode
    #[must_use]
    pub const fn remote_addr(mut self, addr: SocketAddr) -> Self {
        self.config.remote_addr = Some(addr);
        self
    }

    /// Set connection timeout
    #[must_use]
    pub const fn connect_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.connect_timeout_ms = timeout;
        self
    }

    /// Enable or disable keep-alive
    #[must_use]
    pub const fn keep_alive(mut self, enabled: bool) -> Self {
        self.config.keep_alive = enabled;
        self
    }

    /// Set buffer size
    #[must_use]
    pub const fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Build the TCP transport
    #[must_use]
    pub fn build(self) -> TcpTransport {
        if let Some(remote_addr) = self.config.remote_addr {
            TcpTransport::new_client(self.config.bind_addr, remote_addr)
        } else {
            TcpTransport::new_server(self.config.bind_addr)
        }
    }
}

impl Default for TcpTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_config_default() {
        let config = TcpConfig::default();
        assert_eq!(config.bind_addr.to_string(), "127.0.0.1:8080");
        assert_eq!(config.connect_timeout_ms, 5000);
        assert!(config.keep_alive);
    }

    #[test]
    fn test_tcp_transport_builder() {
        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        let transport = TcpTransportBuilder::new()
            .bind_addr(addr)
            .connect_timeout_ms(10000)
            .buffer_size(4096)
            .build();

        assert_eq!(transport.bind_addr, addr);
        assert_eq!(transport.remote_addr, None);
        assert!(matches!(transport.state, TransportState::Disconnected));
    }

    #[test]
    fn test_tcp_transport_client() {
        let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let remote_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let transport = TcpTransportBuilder::new()
            .bind_addr(bind_addr)
            .remote_addr(remote_addr)
            .build();

        assert_eq!(transport.remote_addr, Some(remote_addr));
    }

    #[tokio::test]
    async fn test_tcp_transport_state() {
        let transport = TcpTransportBuilder::new().build();

        assert_eq!(transport.state().await, TransportState::Disconnected);
        assert_eq!(transport.transport_type(), TransportType::Tcp);
    }
}
