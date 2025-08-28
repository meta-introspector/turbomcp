# TurboMCP 1.1.0-exp Release Notes

**Experimental Release - dpop Branch**

This is an experimental release from the `dpop` branch containing the latest TLS transport implementation and security enhancements. This version is intended for testing and feedback before the official 1.1.0 release.

## 🚀 New Features

### Enterprise TLS Transport
- **Complete TLS 1.3/1.2 implementation** with rustls 0.23
- **Certificate pinning** with SHA-256 validation
- **Mutual TLS (mTLS)** support with client certificate authentication
- **OCSP stapling** for real-time certificate validation
- **DPoP integration** for enhanced OAuth 2.0 security (RFC 9449)

### Production Security Features  
- **TLS hardening** with modern cipher suites and security protocols
- **Certificate management** with automatic rotation support
- **Enterprise deployment** configurations for production environments

### Comprehensive Documentation
- **TLS Security Guide** - Production TLS configuration and best practices
- **Deployment Guide** - Docker, Kubernetes, and systemd configurations  
- **Updated transport documentation** with comprehensive TLS examples

## 🔧 Technical Improvements

### Core Architecture
- **Enhanced OAuth2 integration** with fixed import structure
- **Improved error handling** with proper dead code management
- **Production-ready codebase** with zero compilation warnings

### Transport Layer
- **Multi-protocol support**: STDIO, HTTP/SSE, WebSocket, TCP, TLS, Unix sockets
- **Circuit breakers** and fault tolerance mechanisms
- **Performance optimizations** with connection pooling

## 📦 Crate Versions

All crates have been updated to version `1.1.0-exp`:

- `turbomcp` - Main framework crate
- `turbomcp-core` - Core types and SIMD acceleration  
- `turbomcp-protocol` - MCP protocol implementation
- `turbomcp-transport` - Multi-protocol transport with TLS
- `turbomcp-server` - Server framework with OAuth 2.0
- `turbomcp-client` - Client implementation  
- `turbomcp-macros` - Procedural macros
- `turbomcp-cli` - Command-line tools
- `turbomcp-dpop` - DPoP security implementation

## 🔬 Testing Status

- ✅ **All 943+ tests passing** across the workspace
- ✅ **Clippy checks passing** with zero warnings
- ✅ **Compilation verified** with all features
- ✅ **Examples compile** and demonstrate functionality

## 🛡️ Security Enhancements

### TLS Security Features
- TLS 1.3 by default with fallback to TLS 1.2
- Certificate pinning with SHA-256 public key validation
- Mutual TLS support for client authentication
- OCSP stapling for certificate revocation checking
- Enhanced OAuth 2.0 security with DPoP integration

### Production Deployment
- Docker and Kubernetes configuration examples
- Load balancer setup (nginx, HAProxy) with TLS termination
- Certificate management (Let's Encrypt, custom PKI)
- Security hardening and systemd service configuration

## 📚 Documentation Updates

### New Documentation
- `TLS_SECURITY.md` - Comprehensive TLS security guide
- `DEPLOYMENT.md` - Production deployment strategies
- Updated transport README with TLS examples

### Updated Examples
- TLS transport usage examples
- Production configuration samples  
- Security best practices

## 🚨 Breaking Changes

**None** - This is a backward-compatible release with new features.

## ⚠️ Known Issues

This is an experimental release. While thoroughly tested, please report any issues:

1. TLS certificate validation edge cases
2. DPoP implementation compatibility
3. Performance characteristics in production

## 🎯 Migration Guide

### From 1.0.x to 1.1.0-exp

Update your `Cargo.toml`:

```toml
# Old
turbomcp = "1.0.1"

# New  
turbomcp = "1.1.0-exp"
```

### Enabling TLS Transport

```rust
use turbomcp_transport::tls::{TlsTransport, TlsConfig};

let config = TlsConfig::new("server.crt", "server.key");
let server = TlsTransport::new_server("127.0.0.1:8443".parse()?, config).await?;
```

## 🔮 Next Steps

This experimental release helps validate:

1. **TLS implementation completeness**
2. **Production deployment scenarios**
3. **Security feature integration** 
4. **Documentation clarity**

Feedback will be incorporated into the official 1.1.0 release.

## 🙏 Contributing

This experimental release was made possible by:

- Complete TLS transport implementation
- Comprehensive security documentation
- Production deployment guides
- Extensive testing and validation

Report issues or provide feedback through GitHub Issues.

---

**Installation:**

```bash
# From Crates.io (when published)
cargo add turbomcp@1.1.0-exp

# From source (current)
git clone https://github.com/Epistates/turbomcp.git
cd turbomcp
git checkout dpop
cargo build --workspace
```

**⚠️ Experimental Release Notice:**

This version is for testing and feedback. Use in production environments only after thorough testing and validation in your specific use case.