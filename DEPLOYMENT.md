# TurboMCP Deployment Guide

**Production deployment strategies for TurboMCP servers with enterprise security**

## Overview

This guide covers deploying TurboMCP servers in production environments with proper security, monitoring, and scalability considerations. TurboMCP supports multiple transport protocols and deployment patterns to meet various operational requirements.

## Transport Protocols

TurboMCP supports multiple transport protocols for different deployment scenarios:

| Transport | Use Case | Security | Performance |
|-----------|----------|----------|-------------|
| **STDIO** | Local processes, development | Low | Highest |
| **Unix Sockets** | IPC, container communication | Medium | High |
| **TCP** | Network services, internal APIs | Medium | High |
| **TLS** | Encrypted network, production APIs | **High** | Medium |
| **HTTP/SSE** | Web applications, REST APIs | Medium-High | Medium |
| **WebSocket** | Real-time applications, browsers | Medium-High | Medium |

## TLS Deployment (Recommended for Production)

### Basic TLS Setup

```rust
use turbomcp::prelude::*;
use turbomcp_transport::tls::{TlsConfig, TlsVersion};

#[derive(Clone)]
struct ProductionServer;

#[server]
impl ProductionServer {
    #[tool("Health check endpoint")]
    async fn health(&self, ctx: Context) -> McpResult<String> {
        ctx.info("Health check requested").await?;
        Ok("healthy".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Production TLS configuration
    let tls_config = TlsConfig::new("server.crt", "server.key")
        .with_min_version(TlsVersion::V1_3);
    
    let server = ProductionServer;
    server.run_tls("0.0.0.0:8443", tls_config).await?;
    
    Ok(())
}
```

### Enterprise TLS Configuration

```rust
use turbomcp_transport::tls::{
    TlsConfig, CertValidationConfig, CertPinningConfig, ClientAuthMode
};

let enterprise_config = TlsConfig::new("/etc/ssl/certs/server.crt", "/etc/ssl/private/server.key")
    // Security hardening
    .with_min_version(TlsVersion::V1_3)
    .with_mtls() // Mutual TLS for client authentication
    
    // Certificate validation
    .with_cert_validation(CertValidationConfig {
        verify_hostname: true,
        ca_bundle_path: Some("/etc/ssl/certs/ca-bundle.pem".into()),
        client_ca_cert_path: Some("/etc/ssl/certs/client-ca.pem".into()),
        ocsp_stapling: true,
        ct_validation: true,
    })
    
    // Certificate pinning for critical systems
    .with_cert_pinning(CertPinningConfig {
        allowed_hashes: vec![
            "sha256:YLh1dUR9y6Kja30RrAn7JKnbQG/uEtLMkBgFF2Fuihg=".to_string()
        ],
        enforce: true,
    })
    
    // Enhanced OAuth 2.0 security
    .with_dpop_security();
```

## Container Deployment

### Docker Configuration

**Dockerfile:**
```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-turbomcp-server /usr/local/bin/
COPY certs/ /etc/ssl/certs/

EXPOSE 8443
CMD ["my-turbomcp-server"]
```

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  turbomcp-server:
    build: .
    ports:
      - "8443:8443"
    volumes:
      - ./certs:/etc/ssl/certs:ro
      - ./config:/etc/turbomcp:ro
    environment:
      - RUST_LOG=info
      - TRANSPORT=tls
      - TLS_CERT_PATH=/etc/ssl/certs/server.crt
      - TLS_KEY_PATH=/etc/ssl/certs/server.key
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-k", "https://localhost:8443/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Kubernetes Deployment

**turbomcp-deployment.yaml:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: turbomcp-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: turbomcp-server
  template:
    metadata:
      labels:
        app: turbomcp-server
    spec:
      containers:
      - name: turbomcp-server
        image: turbomcp-server:latest
        ports:
        - containerPort: 8443
        env:
        - name: TRANSPORT
          value: "tls"
        - name: TLS_CERT_PATH
          value: "/etc/tls/tls.crt"
        - name: TLS_KEY_PATH
          value: "/etc/tls/tls.key"
        volumeMounts:
        - name: tls-certs
          mountPath: /etc/tls
          readOnly: true
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8443
            scheme: HTTPS
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8443
            scheme: HTTPS
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: tls-certs
        secret:
          secretName: turbomcp-tls
---
apiVersion: v1
kind: Service
metadata:
  name: turbomcp-service
spec:
  selector:
    app: turbomcp-server
  ports:
  - name: https
    port: 443
    targetPort: 8443
  type: LoadBalancer
---
apiVersion: v1
kind: Secret
metadata:
  name: turbomcp-tls
type: kubernetes.io/tls
data:
  tls.crt: # Base64-encoded certificate
  tls.key: # Base64-encoded private key
```

## Load Balancer Configuration

### nginx Configuration

```nginx
upstream turbomcp_backend {
    # Health checks
    server backend1.internal:8443 max_fails=3 fail_timeout=30s;
    server backend2.internal:8443 max_fails=3 fail_timeout=30s;
    server backend3.internal:8443 max_fails=3 fail_timeout=30s;
}

server {
    listen 443 ssl http2;
    server_name api.example.com;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/api.example.com.crt;
    ssl_certificate_key /etc/ssl/private/api.example.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;

    # Security Headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;

    # TurboMCP upstream
    location /mcp {
        proxy_pass https://turbomcp_backend;
        proxy_ssl_verify on;
        proxy_ssl_trusted_certificate /etc/ssl/certs/backend-ca.pem;
        
        # Headers for MCP
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
```

### HAProxy Configuration

```haproxy
global
    ssl-default-bind-ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384
    ssl-default-bind-options ssl-min-ver TLSv1.2 no-tls-tickets

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend turbomcp_frontend
    bind *:443 ssl crt /etc/ssl/certs/api.example.com.pem
    
    # Security headers
    http-response set-header Strict-Transport-Security max-age=31536000
    http-response set-header X-Frame-Options DENY
    http-response set-header X-Content-Type-Options nosniff
    
    default_backend turbomcp_servers

backend turbomcp_servers
    balance roundrobin
    option httpchk GET /health
    http-check expect status 200
    
    server backend1 backend1.internal:8443 check ssl verify required ca-file /etc/ssl/certs/backend-ca.pem
    server backend2 backend2.internal:8443 check ssl verify required ca-file /etc/ssl/certs/backend-ca.pem
    server backend3 backend3.internal:8443 check ssl verify required ca-file /etc/ssl/certs/backend-ca.pem
```

## Certificate Management

### Let's Encrypt with Certbot

```bash
# Install certbot
sudo apt-get update
sudo apt-get install snapd
sudo snap install --classic certbot

# Generate certificates
sudo certbot certonly --standalone -d api.example.com

# Auto-renewal
echo "0 12 * * * /usr/bin/certbot renew --quiet" | sudo crontab -
```

### Custom PKI Setup

```bash
#!/bin/bash
# setup-pki.sh - Internal PKI setup

# Create CA
openssl genrsa -out ca.key 4096
openssl req -new -x509 -days 3650 -key ca.key -out ca.crt \
    -subj "/C=US/ST=CA/L=SF/O=Company/CN=Company Root CA"

# Server certificate
openssl genrsa -out server.key 4096
openssl req -new -key server.key -out server.csr \
    -subj "/C=US/ST=CA/L=SF/O=Company/CN=api.internal"
openssl x509 -req -days 365 -in server.csr -CA ca.crt -CAkey ca.key \
    -CAcreateserial -out server.crt

# Client certificates for mTLS
for client in client1 client2 client3; do
    openssl genrsa -out ${client}.key 4096
    openssl req -new -key ${client}.key -out ${client}.csr \
        -subj "/C=US/ST=CA/L=SF/O=Company/CN=${client}"
    openssl x509 -req -days 365 -in ${client}.csr -CA ca.crt -CAkey ca.key \
        -CAcreateserial -out ${client}.crt
done
```

## Monitoring and Observability

### Prometheus Metrics

```rust
use turbomcp_transport::metrics::{TransportMetrics, MetricsExporter};

// Enable metrics collection
let metrics = TransportMetrics::new()
    .enable_tls_metrics(true)
    .enable_connection_metrics(true)
    .enable_performance_metrics(true);

// Prometheus exporter
let exporter = MetricsExporter::prometheus()
    .bind_address("0.0.0.0:9090")
    .metrics_path("/metrics");

tokio::spawn(async move {
    exporter.serve().await
});
```

**prometheus.yml:**
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'turbomcp'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 10s
```

### Grafana Dashboard

**TurboMCP Dashboard JSON:**
```json
{
  "dashboard": {
    "title": "TurboMCP Server Metrics",
    "panels": [
      {
        "title": "TLS Connections",
        "type": "graph",
        "targets": [
          {
            "expr": "turbomcp_tls_connections_active"
          }
        ]
      },
      {
        "title": "Handshake Duration",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, turbomcp_tls_handshake_duration_seconds)"
          }
        ]
      },
      {
        "title": "Certificate Validation",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(turbomcp_tls_cert_validations_total[5m])"
          }
        ]
      }
    ]
  }
}
```

### Health Checks

```rust
use turbomcp::prelude::*;
use serde_json::json;

#[server]
impl MyServer {
    #[tool("Health check with detailed status")]
    async fn health(&self, ctx: Context) -> McpResult<serde_json::Value> {
        let health_info = json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
            "transport": "tls",
            "tls_version": "1.3",
            "uptime_seconds": self.uptime().as_secs(),
            "memory_usage_mb": self.memory_usage_mb(),
        });
        
        ctx.info("Health check passed").await?;
        Ok(health_info)
    }
}
```

## Security Hardening

### System-Level Security

```bash
# Firewall configuration
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 8443/tcp  # TLS MCP
sudo ufw enable

# User permissions
sudo useradd --system --shell /bin/false turbomcp
sudo mkdir -p /var/lib/turbomcp /var/log/turbomcp
sudo chown turbomcp:turbomcp /var/lib/turbomcp /var/log/turbomcp

# Certificate permissions
sudo chown root:turbomcp /etc/ssl/private/server.key
sudo chmod 640 /etc/ssl/private/server.key
```

### Systemd Service

**/etc/systemd/system/turbomcp.service:**
```ini
[Unit]
Description=TurboMCP Server
After=network.target

[Service]
Type=simple
User=turbomcp
Group=turbomcp
WorkingDirectory=/var/lib/turbomcp
ExecStart=/usr/local/bin/turbomcp-server
Environment=RUST_LOG=info
Environment=TRANSPORT=tls
Environment=TLS_CERT_PATH=/etc/ssl/certs/server.crt
Environment=TLS_KEY_PATH=/etc/ssl/private/server.key

# Security settings
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/turbomcp /var/log/turbomcp

# Resource limits
LimitNOFILE=65536
MemoryHigh=1G
MemoryMax=2G

# Restart policy
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Log Management

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_target(false)
                .json() // Structured logging for production
        )
        .init();
    
    Ok(())
}
```

## Performance Optimization

### Connection Pooling

```rust
use turbomcp_transport::pool::{ConnectionPool, PoolConfig};

let pool_config = PoolConfig::new()
    .max_connections(100)
    .min_connections(10)
    .idle_timeout(Duration::from_secs(300))
    .connection_timeout(Duration::from_secs(10))
    .health_check_interval(Duration::from_secs(30));

let pool = ConnectionPool::new(pool_config);
```

### Circuit Breaker

```rust
use turbomcp_transport::circuit_breaker::{CircuitBreakerConfig, FailureThreshold};

let circuit_config = CircuitBreakerConfig::new()
    .failure_threshold(FailureThreshold::Consecutive(5))
    .recovery_timeout(Duration::from_secs(60))
    .half_open_max_calls(3);
```

### Memory Optimization

```rust
// Environment variables for memory tuning
std::env::set_var("MALLOC_CONF", "background_thread:true,metadata_thp:auto");

// Tokio runtime optimization
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)  // Match CPU cores
    .thread_stack_size(2 * 1024 * 1024)  // 2MB stack
    .enable_all()
    .build()?;
```

## Multi-Environment Configuration

### Environment-Specific Configs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub transport: TransportType,
    pub tls: TlsConfiguration,
    pub security: SecurityConfiguration,
    pub monitoring: MonitoringConfiguration,
}

impl ServerConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        let config_path = match env.as_str() {
            "production" => "config/production.toml",
            "staging" => "config/staging.toml",
            _ => "config/development.toml",
        };
        
        let config_content = std::fs::read_to_string(config_path)?;
        let config: ServerConfig = toml::from_str(&config_content)?;
        
        Ok(config)
    }
}
```

**config/production.toml:**
```toml
[transport]
type = "tls"
bind_address = "0.0.0.0:8443"

[tls]
cert_path = "/etc/ssl/certs/server.crt"
key_path = "/etc/ssl/private/server.key"
min_version = "1.3"
enable_mtls = true
enable_ocsp_stapling = true
enable_dpop_security = true

[security]
enable_certificate_pinning = true
allowed_certificate_hashes = [
    "sha256:YLh1dUR9y6Kja30RrAn7JKnbQG/uEtLMkBgFF2Fuihg="
]

[monitoring]
enable_metrics = true
metrics_port = 9090
enable_health_checks = true
```

## Testing Deployments

### Integration Tests

```rust
#[cfg(test)]
mod deployment_tests {
    use super::*;
    use turbomcp_transport::tls::{TlsTransport, TlsConfig};
    
    #[tokio::test]
    async fn test_tls_deployment() {
        // Test certificate loading
        let config = TlsConfig::new("tests/certs/server.crt", "tests/certs/server.key");
        
        // Test server creation
        let server = TlsTransport::new_server("127.0.0.1:8443".parse().unwrap(), config).await;
        assert!(server.is_ok());
        
        // Test client connection
        let client_config = TlsConfig::new("tests/certs/client.crt", "tests/certs/client.key");
        let client = TlsTransport::new_client("127.0.0.1:8443".parse().unwrap(), client_config).await;
        assert!(client.is_ok());
    }
}
```

### Smoke Tests

```bash
#!/bin/bash
# smoke-test.sh

set -e

echo "Starting smoke tests for TurboMCP deployment..."

# Test TLS connectivity
echo "Testing TLS connectivity..."
curl -k --max-time 10 https://localhost:8443/health

# Test certificate validation
echo "Testing certificate validation..."
openssl s_client -connect localhost:8443 -servername localhost < /dev/null

# Test mTLS if enabled
if [[ "$MTLS_ENABLED" == "true" ]]; then
    echo "Testing mutual TLS..."
    curl --cert client.crt --key client.key --cacert ca.crt \
         https://localhost:8443/health
fi

echo "Smoke tests completed successfully!"
```

## Troubleshooting

### Common Issues

1. **Certificate Problems**
   ```bash
   # Check certificate validity
   openssl x509 -in server.crt -text -noout | grep -A 2 Validity
   
   # Verify certificate chain
   openssl verify -CAfile ca.crt server.crt
   ```

2. **TLS Handshake Failures**
   ```bash
   # Test TLS connection
   openssl s_client -connect localhost:8443 -debug -msg
   ```

3. **Performance Issues**
   ```bash
   # Monitor connections
   ss -tuln | grep :8443
   
   # Check resource usage
   top -p $(pgrep turbomcp-server)
   ```

## Scaling Considerations

### Horizontal Scaling

```yaml
# kubernetes/horizontal-pod-autoscaler.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: turbomcp-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: turbomcp-server
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Vertical Scaling

```rust
// Resource-aware server configuration
let server_config = match get_available_memory() {
    mem if mem > 8_000_000_000 => ServerConfig::high_memory(),
    mem if mem > 4_000_000_000 => ServerConfig::medium_memory(),
    _ => ServerConfig::low_memory(),
};
```

This deployment guide provides comprehensive coverage of production TurboMCP deployments with TLS security. For specific use cases or additional support, consult the [TLS Security Guide](./crates/turbomcp-transport/TLS_SECURITY.md).

---

*For more information, see the [TurboMCP Documentation](./README.md) and [Security Features](./crates/turbomcp-transport/SECURITY_FEATURES.md).*