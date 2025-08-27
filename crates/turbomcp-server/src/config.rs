//! Server configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server description
    pub description: Option<String>,
    /// Bind address
    pub bind_address: String,
    /// Bind port
    pub port: u16,
    /// Enable TLS
    pub enable_tls: bool,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// Timeout configuration
    pub timeouts: TimeoutConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Additional configuration
    pub additional: HashMap<String, serde_json::Value>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: PathBuf,
    /// Private key file path
    pub key_file: PathBuf,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Request timeout
    pub request_timeout: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second
    pub requests_per_second: u32,
    /// Burst capacity
    pub burst_capacity: u32,
}

/// Security configuration  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security middleware
    pub enabled: bool,
    /// Rate limiting strategy
    pub rate_limiting_strategy: RateLimitingStrategy,
    /// DoS protection configuration
    pub dos_protection: DoSProtectionConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Security event logging
    pub event_logging: bool,
    /// Trusted IP addresses (bypass all security)
    pub trusted_ips: Vec<IpAddr>,
    /// DPoP configuration
    pub dpop: Option<DpopSecurityConfig>,
}

/// Rate limiting strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitingStrategy {
    /// Per-IP rate limiting
    PerIp {
        /// Requests allowed per second
        requests_per_second: u32,
        /// Burst capacity for traffic spikes
        burst_capacity: u32,
    },
    /// Per-DPoP key rate limiting  
    PerDpopKey {
        /// Requests allowed per second
        requests_per_second: u32,
        /// Burst capacity for traffic spikes
        burst_capacity: u32,
    },
    /// Combined IP + DPoP key limiting
    Combined {
        /// IP-based requests per second
        ip_requests_per_second: u32,
        /// IP-based burst capacity
        ip_burst_capacity: u32,
        /// DPoP key-based requests per second
        dpop_requests_per_second: u32,
        /// DPoP key-based burst capacity
        dpop_burst_capacity: u32,
    },
    /// Adaptive rate limiting based on load
    Adaptive {
        /// Base requests per second under normal load
        base_requests_per_second: u32,
        /// Maximum requests per second during high load
        max_requests_per_second: u32,
    },
}

/// DoS protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoSProtectionConfig {
    /// Enable DoS protection
    pub enabled: bool,
    /// Maximum requests per minute before considering DoS
    pub max_requests_per_minute: u32,
    /// Block duration for suspicious IPs (in seconds)
    pub block_duration_seconds: u64,
    /// Suspicious activity threshold
    pub suspicious_threshold: u32,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breakers
    pub enabled: bool,
    /// Failure threshold (percentage)
    pub failure_threshold: f32,
    /// Recovery time (in seconds)
    pub recovery_time_seconds: u64,
    /// Request volume threshold
    pub request_volume_threshold: u32,
}

/// DPoP security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpopSecurityConfig {
    /// Enable DPoP key tracking
    pub key_tracking: bool,
    /// Maximum requests per DPoP key per minute
    pub max_requests_per_key_per_minute: u32,
    /// Enable DPoP key rotation detection
    pub key_rotation_detection: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Enable structured logging
    pub structured: bool,
    /// Log file path
    pub file: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: crate::SERVER_NAME.to_string(),
            version: crate::SERVER_VERSION.to_string(),
            description: Some("Next generation MCP server".to_string()),
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            enable_tls: false,
            tls: None,
            timeouts: TimeoutConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            additional: HashMap::new(),
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            keep_alive_timeout: Duration::from_secs(60),
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_second: 100,
            burst_capacity: 200,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rate_limiting_strategy: RateLimitingStrategy::PerIp {
                requests_per_second: 100,
                burst_capacity: 200,
            },
            dos_protection: DoSProtectionConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            event_logging: true,
            trusted_ips: Vec::new(),
            dpop: None, // DPoP security disabled by default
        }
    }
}

impl Default for DoSProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_minute: 300,
            block_duration_seconds: 3600, // 1 hour
            suspicious_threshold: 50,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 0.5, // 50% failure rate
            recovery_time_seconds: 60,
            request_volume_threshold: 10,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            structured: true,
            file: None,
        }
    }
}

/// Configuration builder
#[derive(Debug)]
pub struct ConfigurationBuilder {
    /// Configuration being built
    config: ServerConfig,
}

impl ConfigurationBuilder {
    /// Create a new configuration builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }

    /// Set server name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set server version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = version.into();
        self
    }

    /// Set server description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.config.description = Some(description.into());
        self
    }

    /// Set bind address
    pub fn bind_address(mut self, address: impl Into<String>) -> Self {
        self.config.bind_address = address.into();
        self
    }

    /// Set port
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Enable TLS with configuration
    #[must_use]
    pub fn tls(mut self, cert_file: PathBuf, key_file: PathBuf) -> Self {
        self.config.enable_tls = true;
        self.config.tls = Some(TlsConfig {
            cert_file,
            key_file,
        });
        self
    }

    /// Set request timeout
    #[must_use]
    pub const fn request_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeouts.request_timeout = timeout;
        self
    }

    /// Enable rate limiting
    #[must_use]
    pub const fn rate_limiting(mut self, requests_per_second: u32, burst_capacity: u32) -> Self {
        self.config.rate_limiting.enabled = true;
        self.config.rate_limiting.requests_per_second = requests_per_second;
        self.config.rate_limiting.burst_capacity = burst_capacity;
        self
    }

    /// Set log level
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.config.logging.level = level.into();
        self
    }

    /// Enable security middleware
    #[must_use]
    pub const fn enable_security(mut self, enabled: bool) -> Self {
        self.config.security.enabled = enabled;
        self
    }

    /// Set security rate limiting strategy
    #[must_use]
    pub fn security_rate_limiting(mut self, strategy: RateLimitingStrategy) -> Self {
        self.config.security.rate_limiting_strategy = strategy;
        self
    }

    /// Configure DoS protection
    #[must_use]
    pub fn dos_protection(mut self, config: DoSProtectionConfig) -> Self {
        self.config.security.dos_protection = config;
        self
    }

    /// Configure circuit breaker
    #[must_use]
    pub fn circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.config.security.circuit_breaker = config;
        self
    }

    /// Add trusted IP addresses
    pub fn trusted_ips(mut self, ips: Vec<IpAddr>) -> Self {
        self.config.security.trusted_ips = ips;
        self
    }

    /// Enable DPoP security features
    #[must_use]
    pub fn dpop_security(mut self, config: DpopSecurityConfig) -> Self {
        self.config.security.dpop = Some(config);
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> ServerConfig {
        self.config
    }
}

impl Default for ConfigurationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration alias for convenience
pub type Configuration = ServerConfig;
