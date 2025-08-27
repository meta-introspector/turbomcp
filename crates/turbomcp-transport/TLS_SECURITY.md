# TLS Security Guide for TurboMCP Transport

**Production-grade TLS implementation with enterprise security features**

## Overview

TurboMCP's TLS transport provides state-of-the-art security for MCP communications using the `rustls` library, which offers memory-safe TLS 1.3 implementation with production-ready security features.

## Security Features

### 🔐 **TLS 1.3 by Default**
- **Latest TLS Version**: TLS 1.3 with automatic fallback to TLS 1.2
- **Forward Secrecy**: All connections use perfect forward secrecy
- **Modern Cipher Suites**: AEAD-only cipher suites (AES-GCM, ChaCha20-Poly1305)
- **0-RTT Prevention**: Built-in protection against replay attacks

### 🔑 **Certificate Management**
- **X.509 Certificate Support**: Standard PKI infrastructure
- **Certificate Chain Validation**: Full chain verification with intermediate CAs  
- **OCSP Stapling**: Real-time revocation status checking
- **Certificate Transparency**: Optional CT log validation
- **Certificate Pinning**: Pin specific certificates or public keys

### 🤝 **Mutual TLS (mTLS)**
- **Client Certificate Authentication**: Verify client identity
- **Flexible Authentication Modes**: None, Optional, or Required
- **Custom CA Support**: Use your own certificate authority
- **Certificate-based Authorization**: Map certificates to permissions

### 🛡️ **Enhanced Security Features**
- **DPoP Integration**: RFC 9449 Demonstration of Proof-of-Possession
- **Memory Safety**: Rust's memory safety prevents common TLS vulnerabilities
- **Constant-Time Operations**: Timing attack resistance
- **Session Management**: Secure session resumption and rotation

## Quick Start

### Basic TLS Server

```rust
use turbomcp_transport::tls::{TlsTransport, TlsConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TlsConfig::new("server.crt", "server.key");
    let addr = "127.0.0.1:8443".parse::<SocketAddr>()?;
    
    let server = TlsTransport::new_server(addr, config).await?;
    
    // Server is ready to accept secure connections
    println!("TLS server listening on {}", addr);
    
    Ok(())
}
```

### Basic TLS Client

```rust
use turbomcp_transport::tls::{TlsTransport, TlsConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TlsConfig::new("client.crt", "client.key");
    let addr = "api.example.com:8443".parse::<SocketAddr>()?;
    
    let client = TlsTransport::new_client(addr, config).await?;
    
    // Client is connected securely
    println!("Connected to TLS server at {}", addr);
    
    Ok(())
}
```

## Production Configuration

### Enterprise TLS Server

```rust
use turbomcp_transport::tls::{
    TlsTransport, TlsConfig, CertValidationConfig, CertPinningConfig,
    ClientAuthMode, TlsVersion
};

let tls_config = TlsConfig::new("server.crt", "server.key")
    // Security settings
    .with_min_version(TlsVersion::V1_3)
    .with_mtls()
    
    // Certificate validation
    .with_cert_validation(CertValidationConfig {
        verify_hostname: true,
        ca_bundle_path: Some("/etc/ssl/certs/ca-bundle.pem".into()),
        client_ca_cert_path: Some("/etc/ssl/certs/client-ca.pem".into()),
        ocsp_stapling: true,
        ct_validation: true,
    })
    
    // Certificate pinning for high-security environments
    .with_cert_pinning(CertPinningConfig {
        allowed_hashes: vec![
            "sha256:YLh1dUR9y6Kja30RrAn7JKnbQG/uEtLMkBgFF2Fuihg=".to_string(),
            "sha256:C5+lpZ7tcVwmwQIMcRtPbsQtWLABXhQzejna0wHFr8M=".to_string(),
        ],
        enforce: true,
    })
    
    // Enhanced OAuth 2.0 security
    .with_dpop_security();

let server = TlsTransport::new_server("0.0.0.0:8443".parse()?, tls_config).await?;
```

### Load Balancer Setup

```rust
use turbomcp_transport::tls::{TlsConfig, TlsTransport};

// Backend servers with TLS termination
let backend_configs = vec![
    ("backend1.internal:8443", TlsConfig::new("backend1.crt", "backend1.key")),
    ("backend2.internal:8443", TlsConfig::new("backend2.crt", "backend2.key")),
    ("backend3.internal:8443", TlsConfig::new("backend3.crt", "backend3.key")),
];

for (addr, config) in backend_configs {
    let server = TlsTransport::new_server(addr.parse()?, config).await?;
    tokio::spawn(async move {
        // Handle server requests
    });
}
```

## Certificate Management

### Certificate Generation

For development and testing:

```bash
# Generate CA private key
openssl genrsa -out ca.key 4096

# Generate CA certificate
openssl req -new -x509 -days 365 -key ca.key -out ca.crt \
    -subj "/C=US/ST=CA/L=San Francisco/O=TurboMCP/CN=TurboMCP CA"

# Generate server private key
openssl genrsa -out server.key 4096

# Generate server certificate signing request
openssl req -new -key server.key -out server.csr \
    -subj "/C=US/ST=CA/L=San Francisco/O=TurboMCP/CN=localhost"

# Sign server certificate
openssl x509 -req -days 365 -in server.csr -CA ca.crt -CAkey ca.key \
    -CAcreateserial -out server.crt

# Generate client certificate (for mTLS)
openssl genrsa -out client.key 4096
openssl req -new -key client.key -out client.csr \
    -subj "/C=US/ST=CA/L=San Francisco/O=TurboMCP Client/CN=client"
openssl x509 -req -days 365 -in client.csr -CA ca.crt -CAkey ca.key \
    -CAcreateserial -out client.crt
```

### Certificate Rotation

```rust
use turbomcp_transport::tls::{TlsConfig, CertificateManager};
use std::time::Duration;

// Automatic certificate rotation
let cert_manager = CertificateManager::new()
    .cert_path("/etc/ssl/certs/server.crt")
    .key_path("/etc/ssl/private/server.key")
    .check_interval(Duration::from_secs(3600)) // Check every hour
    .rotation_threshold(Duration::from_days(30)); // Rotate 30 days before expiry

let config = TlsConfig::with_manager(cert_manager)
    .with_min_version(TlsVersion::V1_3);
```

## Security Best Practices

### 1. Certificate Security

```rust
use turbomcp_transport::tls::{TlsConfig, CertValidationConfig};

let config = TlsConfig::new("server.crt", "server.key")
    .with_cert_validation(CertValidationConfig {
        // Always verify hostnames in production
        verify_hostname: true,
        
        // Use custom CA bundle for internal PKI
        ca_bundle_path: Some("/etc/ssl/certs/internal-ca.pem".into()),
        
        // Enable OCSP stapling for revocation checking
        ocsp_stapling: true,
        
        // Enable Certificate Transparency validation
        ct_validation: true,
    });
```

### 2. mTLS Configuration

```rust
use turbomcp_transport::tls::{TlsConfig, ClientAuthMode};

// Server configuration for mTLS
let server_config = TlsConfig::new("server.crt", "server.key")
    .with_client_auth(ClientAuthMode::Required) // Require client certificates
    .with_cert_validation(CertValidationConfig {
        client_ca_cert_path: Some("/etc/ssl/certs/client-ca.pem".into()),
        verify_hostname: true,
        ocsp_stapling: true,
        ct_validation: false, // Often disabled for client certs
    });

// Client configuration
let client_config = TlsConfig::new("client.crt", "client.key")
    .with_cert_validation(CertValidationConfig {
        ca_bundle_path: Some("/etc/ssl/certs/server-ca.pem".into()),
        verify_hostname: true,
        ocsp_stapling: false, // Client doesn't provide OCSP
        ct_validation: false,
    });
```

### 3. Certificate Pinning

```rust
use turbomcp_transport::tls::{TlsConfig, CertPinningConfig};

// High-security environments with certificate pinning
let config = TlsConfig::new("server.crt", "server.key")
    .with_cert_pinning(CertPinningConfig {
        // SHA-256 hashes of allowed public keys
        allowed_hashes: vec![
            // Primary certificate
            "sha256:YLh1dUR9y6Kja30RrAn7JKnbQG/uEtLMkBgFF2Fuihg=".to_string(),
            // Backup certificate (for rotation)
            "sha256:C5+lpZ7tcVwmwQIMcRtPbsQtWLABXhQzejna0wHFr8M=".to_string(),
        ],
        enforce: true, // Fail connections on pin mismatch
    });
```

## DPoP Integration

### Enhanced OAuth 2.0 Security

```rust
use turbomcp_transport::tls::TlsConfig;
use turbomcp_dpop::{DpopKeyManager, DpopProofGenerator};

// Enable DPoP for enhanced OAuth 2.0 security
let tls_config = TlsConfig::new("server.crt", "server.key")
    .with_dpop_security(); // Automatically configures TLS 1.3 and security headers

// Generate DPoP proof for API requests
let key_manager = DpopKeyManager::new_memory().await?;
let proof_generator = DpopProofGenerator::new(key_manager.into());

let proof = proof_generator.generate_proof(
    "POST",
    "https://api.example.com/mcp",
    Some("access_token_here"),
).await?;

// Use proof in HTTP headers
// DPoP: <proof_jwt>
// Authorization: Bearer <access_token>
```

## Troubleshooting

### Common TLS Issues

#### 1. Certificate Verification Failures

```rust
// Debug certificate issues
use turbomcp_transport::tls::{TlsError, TlsConfig};

let config = TlsConfig::new("server.crt", "server.key");

match TlsTransport::new_server("127.0.0.1:8443".parse()?, config).await {
    Err(TlsError::Certificate { reason }) => {
        eprintln!("Certificate error: {}", reason);
        // Check certificate file permissions
        // Verify certificate format (PEM)
        // Ensure certificate and key match
    }
    Err(TlsError::Configuration { reason }) => {
        eprintln!("Configuration error: {}", reason);
        // Check TLS configuration parameters
        // Verify file paths exist
    }
    Ok(transport) => {
        println!("TLS transport created successfully");
    }
}
```

#### 2. Handshake Failures

```bash
# Test TLS connection with OpenSSL
openssl s_client -connect localhost:8443 -servername localhost

# Verify certificate
openssl x509 -in server.crt -text -noout

# Check private key
openssl rsa -in server.key -check
```

#### 3. Certificate Pinning Issues

```rust
use turbomcp_transport::tls::{TlsConfig, CertPinningConfig};

// Generate certificate hash for pinning
use sha2::{Sha256, Digest};
use std::fs;

let cert_der = fs::read("server.crt")?;
let mut hasher = Sha256::new();
hasher.update(&cert_der);
let hash = format!("sha256:{}", base64::encode(hasher.finalize()));
println!("Certificate hash: {}", hash);

// Use in pinning configuration
let config = TlsConfig::new("server.crt", "server.key")
    .with_cert_pinning(CertPinningConfig {
        allowed_hashes: vec![hash],
        enforce: false, // Set to false for testing
    });
```

## Performance Optimization

### TLS Performance Tuning

```rust
use turbomcp_transport::tls::{TlsConfig, TlsVersion};

let config = TlsConfig::new("server.crt", "server.key")
    // Use TLS 1.3 for better performance
    .with_min_version(TlsVersion::V1_3)
    
    // Enable session resumption
    .with_session_resumption(true)
    
    // Configure session cache
    .with_session_cache_size(1024)
    
    // Set session timeout
    .with_session_timeout(Duration::from_secs(3600));
```

### Connection Pooling

```rust
use turbomcp_transport::tls::{TlsTransport, TlsConfig, ConnectionPool};

let config = TlsConfig::new("client.crt", "client.key");

// Create connection pool for clients
let pool = ConnectionPool::new()
    .max_connections(100)
    .idle_timeout(Duration::from_secs(300))
    .connection_timeout(Duration::from_secs(10));

let transport = TlsTransport::new_client_pooled("api.example.com:8443".parse()?, config, pool).await?;
```

## Monitoring and Observability

### TLS Metrics

```rust
use turbomcp_transport::tls::{TlsTransport, TlsMetrics};

// Collect TLS-specific metrics
let metrics = transport.tls_metrics().await;

println!("TLS Version: {:?}", metrics.tls_version);
println!("Cipher Suite: {}", metrics.cipher_suite);
println!("Handshake Duration: {:?}", metrics.handshake_duration);
println!("Certificates Validated: {}", metrics.certs_validated);
println!("OCSP Checks: {}", metrics.ocsp_checks);
println!("Session Resumptions: {}", metrics.session_resumptions);
```

### Security Events

```rust
use turbomcp_transport::tls::{TlsSecurityEvent, SecurityEventHandler};

// Monitor security events
struct SecurityMonitor;

impl SecurityEventHandler for SecurityMonitor {
    fn on_certificate_pinning_failure(&self, event: &TlsSecurityEvent) {
        log::error!("Certificate pinning failed: {:?}", event);
        // Alert security team
    }
    
    fn on_handshake_failure(&self, event: &TlsSecurityEvent) {
        log::warn!("TLS handshake failed: {:?}", event);
        // Track failed attempts
    }
    
    fn on_client_auth_failure(&self, event: &TlsSecurityEvent) {
        log::warn!("Client authentication failed: {:?}", event);
        // Monitor for attacks
    }
}
```

## Compliance and Standards

### Industry Standards

- **FIPS 140-2**: Cryptographic module validation (via `rustls` and `ring`)
- **Common Criteria**: Security evaluation standards
- **NIST Guidelines**: Following NIST SP 800-52 Rev. 2
- **RFC Compliance**: TLS 1.3 (RFC 8446), X.509 (RFC 5280)

### Regulatory Compliance

```rust
use turbomcp_transport::tls::{TlsConfig, ComplianceLevel};

// HIPAA-compliant configuration
let hipaa_config = TlsConfig::compliance(ComplianceLevel::HIPAA)
    .with_min_version(TlsVersion::V1_3) // Required
    .with_mtls() // Client authentication required
    .with_audit_logging(true); // Log all TLS events

// PCI DSS-compliant configuration
let pci_config = TlsConfig::compliance(ComplianceLevel::PCIDSS)
    .with_min_version(TlsVersion::V1_2) // Minimum requirement
    .with_strong_ciphers_only(true) // No weak ciphers
    .with_certificate_validation_required(true);
```

## Migration Guide

### From HTTP to TLS

```rust
// Before: HTTP transport
use turbomcp_transport::http::HttpTransport;
let http_transport = HttpTransport::new("127.0.0.1:8080").await?;

// After: TLS transport
use turbomcp_transport::tls::{TlsTransport, TlsConfig};
let tls_config = TlsConfig::new("server.crt", "server.key");
let tls_transport = TlsTransport::new_server("127.0.0.1:8443".parse()?, tls_config).await?;
```

### TLS Version Migration

```rust
// Gradual migration from TLS 1.2 to TLS 1.3
let config = TlsConfig::new("server.crt", "server.key")
    .with_min_version(TlsVersion::V1_2) // Start with 1.2
    .with_preferred_version(TlsVersion::V1_3); // Prefer 1.3

// After testing, upgrade to TLS 1.3 only
let strict_config = TlsConfig::new("server.crt", "server.key")
    .with_min_version(TlsVersion::V1_3); // TLS 1.3 only
```

## Related Documentation

- **[Transport README](./README.md)** - General transport layer documentation
- **[Security Features Guide](./SECURITY_FEATURES.md)** - Comprehensive security features
- **[DPoP Documentation](../turbomcp-dpop/README.md)** - OAuth 2.0 DPoP implementation
- **[rustls Documentation](https://docs.rs/rustls/)** - TLS implementation details

---

*TurboMCP TLS Transport provides enterprise-grade security for production MCP deployments. For support, see our [GitHub repository](https://github.com/Epistates/turbomcp).*