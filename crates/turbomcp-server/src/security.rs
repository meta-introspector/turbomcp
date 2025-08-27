//! Enterprise Security Middleware for TurboMCP
//!
//! This module provides production-grade security features including:
//! - **Token bucket rate limiting** with multiple algorithms
//! - **IP-based DoS protection** with automatic blocking
//! - **DPoP-aware rate limiting** for enhanced security
//! - **Circuit breaker patterns** for service protection
//! - **Comprehensive metrics and monitoring**

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::config::{CircuitBreakerConfig, RateLimitingStrategy, SecurityConfig};
use crate::error::ServerError;
use crate::middleware::Middleware;
use turbomcp_core::RequestContext;
use turbomcp_protocol::jsonrpc::{JsonRpcRequest, JsonRpcResponse};

/// Security middleware errors
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {reason}")]
    RateLimitExceeded {
        /// Reason for rate limit being exceeded
        reason: String,
    },

    /// IP address blocked due to abuse
    #[error("IP address blocked: {reason}")]
    IpBlocked {
        /// Reason for IP being blocked
        reason: String,
    },

    /// Circuit breaker is open
    #[error("Circuit breaker open: {reason}")]
    CircuitBreakerOpen {
        /// Reason for circuit breaker being open
        reason: String,
    },

    /// Configuration error
    #[error("Security configuration error: {reason}")]
    Configuration {
        /// Configuration error details
        reason: String,
    },

    /// DPoP security violation
    #[error("DPoP security violation: {reason}")]
    DpopViolation {
        /// Details of the DPoP violation
        reason: String,
    },
}

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Rate limiting algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    /// Token bucket algorithm (most flexible)
    TokenBucket,
    /// Fixed window counter
    FixedWindow,
    /// Sliding window log
    SlidingWindowLog,
    /// Sliding window counter (memory efficient)
    SlidingWindowCounter,
}

impl Default for RateLimitAlgorithm {
    fn default() -> Self {
        Self::TokenBucket // Best balance of accuracy and performance
    }
}

/// Security event types for monitoring
#[derive(Debug, Clone, Serialize)]
pub enum SecurityEvent {
    /// Rate limit exceeded
    RateLimitExceeded {
        /// IP address that exceeded rate limit
        ip: IpAddr,
        /// Endpoint that was rate limited
        endpoint: String,
        /// Current request rate
        current_rate: u32,
        /// Configured rate limit
        limit: u32,
    },
    /// IP blocked due to abuse
    IpBlocked {
        /// Blocked IP address
        ip: IpAddr,
        /// Reason for blocking
        reason: String,
        /// Duration of the block
        duration: Duration,
    },
    /// Suspicious activity detected
    SuspiciousActivity {
        /// IP address showing suspicious activity
        ip: IpAddr,
        /// Pattern of suspicious activity
        pattern: String,
        /// Severity score (1-10 scale)
        severity: u8,
    },
    /// Circuit breaker opened
    CircuitBreakerOpened {
        /// Service with opened circuit breaker
        service: String,
        /// Number of failures that triggered the opening
        failure_count: u32,
    },
    /// DPoP security violation
    DpopViolation {
        /// Type of DPoP violation
        violation_type: String,
        /// Additional violation details
        details: String,
    },
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = (elapsed * self.refill_rate) as u32;

        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill = now;
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Blocking all requests
    HalfOpen, // Testing recovery
}

/// Circuit breaker implementation
#[derive(Debug)]
struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            config,
        }
    }

    fn call_permitted(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed()
                        >= Duration::from_secs(self.config.recovery_time_seconds)
                    {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= 3 {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitBreakerState::Closed => {
                self.failure_count = 0; // Reset failures on success
            }
            _ => {}
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.config.failure_threshold as u32 {
            self.state = CircuitBreakerState::Open;
        }
    }
}

/// Request context for security middleware
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Client IP address
    pub ip: IpAddr,
    /// Request endpoint/path
    pub endpoint: String,
    /// DPoP key thumbprint (if present)
    pub dpop_thumbprint: Option<String>,
    /// Request timestamp
    pub timestamp: SystemTime,
    /// Request ID for tracing
    pub request_id: String,
}

/// Security middleware implementation
pub struct SecurityMiddleware {
    config: Arc<SecurityConfig>,

    // Rate limiting state
    ip_buckets: Arc<RwLock<HashMap<IpAddr, TokenBucket>>>,
    dpop_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,

    // DoS protection state
    blocked_ips: Arc<RwLock<HashMap<IpAddr, Instant>>>,
    suspicious_ips: Arc<RwLock<HashMap<IpAddr, (u32, Instant)>>>,

    // Circuit breaker state
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,

    // Security events
    event_sender: Option<tokio::sync::mpsc::UnboundedSender<SecurityEvent>>,
}

impl std::fmt::Debug for SecurityMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecurityMiddleware")
            .field("config", &self.config)
            .field("ip_buckets_count", &self.ip_buckets.read().len())
            .field("dpop_buckets_count", &self.dpop_buckets.read().len())
            .field("blocked_ips_count", &self.blocked_ips.read().len())
            .field("suspicious_ips_count", &self.suspicious_ips.read().len())
            .field(
                "circuit_breakers_count",
                &self.circuit_breakers.read().len(),
            )
            .field("has_event_sender", &self.event_sender.is_some())
            .finish()
    }
}

impl SecurityMiddleware {
    /// Create a new security middleware instance
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config: Arc::new(config),
            ip_buckets: Arc::new(RwLock::new(HashMap::new())),
            dpop_buckets: Arc::new(RwLock::new(HashMap::new())),
            blocked_ips: Arc::new(RwLock::new(HashMap::new())),
            suspicious_ips: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            event_sender: None,
        }
    }

    /// Create middleware with event monitoring
    pub fn with_event_monitoring(
        config: SecurityConfig,
        event_sender: tokio::sync::mpsc::UnboundedSender<SecurityEvent>,
    ) -> Self {
        let mut middleware = Self::new(config);
        middleware.event_sender = Some(event_sender);
        middleware
    }

    /// Check if request should be allowed through security middleware
    pub async fn check_request(&self, ctx: &SecurityContext) -> SecurityResult<()> {
        // 1. Check if IP is blocked
        self.check_ip_blocked(ctx).await?;

        // 2. Check rate limits
        self.check_rate_limits(ctx).await?;

        // 3. Check circuit breakers
        self.check_circuit_breakers(ctx).await?;

        // 4. Check DPoP-specific security (if enabled and present)
        if let Some(dpop_config) = &self.config.dpop
            && dpop_config.key_tracking
            && ctx.dpop_thumbprint.is_some()
        {
            self.check_dpop_security(ctx).await?;
        }

        // 5. Update suspicious activity monitoring
        self.update_activity_monitoring(ctx).await;

        Ok(())
    }

    /// Record successful request completion
    pub async fn record_success(&self, ctx: &SecurityContext) {
        if self.config.circuit_breaker.enabled {
            let mut breakers = self.circuit_breakers.write();
            if let Some(breaker) = breakers.get_mut(&ctx.endpoint) {
                breaker.record_success();
            }
        }
    }

    /// Record failed request
    pub async fn record_failure(&self, ctx: &SecurityContext, error: &str) {
        if self.config.circuit_breaker.enabled {
            let (should_emit, service, failure_count) = {
                let mut breakers = self.circuit_breakers.write();
                let breaker = breakers
                    .entry(ctx.endpoint.clone())
                    .or_insert_with(|| CircuitBreaker::new(self.config.circuit_breaker.clone()));

                breaker.record_failure();

                (
                    breaker.state == CircuitBreakerState::Open,
                    ctx.endpoint.clone(),
                    breaker.failure_count,
                )
            };

            if should_emit {
                let _ = self
                    .emit_event(SecurityEvent::CircuitBreakerOpened {
                        service,
                        failure_count,
                    })
                    .await;
            }
        }

        if self.config.event_logging {
            warn!(
                request_id = %ctx.request_id,
                ip = %ctx.ip,
                endpoint = %ctx.endpoint,
                error = %error,
                "Request failed"
            );
        }
    }

    /// Check if IP address is blocked
    async fn check_ip_blocked(&self, ctx: &SecurityContext) -> SecurityResult<()> {
        if !self.config.dos_protection.enabled {
            return Ok(());
        }

        let blocked_ips = self.blocked_ips.read();
        if let Some(&blocked_until) = blocked_ips.get(&ctx.ip)
            && blocked_until > Instant::now()
        {
            return Err(SecurityError::IpBlocked {
                reason: "IP temporarily blocked due to abuse".to_string(),
            });
        }
        Ok(())
    }

    /// Check rate limits
    async fn check_rate_limits(&self, ctx: &SecurityContext) -> SecurityResult<()> {
        match &self.config.rate_limiting_strategy {
            RateLimitingStrategy::PerIp {
                requests_per_second,
                burst_capacity,
            } => {
                self.check_ip_rate_limit(ctx, *requests_per_second, *burst_capacity)
                    .await?;
            }
            RateLimitingStrategy::PerDpopKey {
                requests_per_second,
                burst_capacity,
            } => {
                if let Some(thumbprint) = &ctx.dpop_thumbprint {
                    self.check_dpop_rate_limit(
                        ctx,
                        thumbprint,
                        *requests_per_second,
                        *burst_capacity,
                    )
                    .await?;
                }
            }
            RateLimitingStrategy::Combined {
                ip_requests_per_second,
                ip_burst_capacity,
                dpop_requests_per_second,
                dpop_burst_capacity,
            } => {
                self.check_ip_rate_limit(ctx, *ip_requests_per_second, *ip_burst_capacity)
                    .await?;
                if let Some(thumbprint) = &ctx.dpop_thumbprint {
                    self.check_dpop_rate_limit(
                        ctx,
                        thumbprint,
                        *dpop_requests_per_second,
                        *dpop_burst_capacity,
                    )
                    .await?;
                }
            }
            RateLimitingStrategy::Adaptive {
                base_requests_per_second,
                max_requests_per_second: _,
            } => {
                // For now, use base rate - adaptive logic can be enhanced later
                self.check_ip_rate_limit(
                    ctx,
                    *base_requests_per_second,
                    base_requests_per_second * 2,
                )
                .await?;
            }
        }
        Ok(())
    }

    /// Check IP-based rate limiting
    async fn check_ip_rate_limit(
        &self,
        ctx: &SecurityContext,
        requests_per_second: u32,
        burst_capacity: u32,
    ) -> SecurityResult<()> {
        let refill_rate = requests_per_second as f64;
        let capacity = burst_capacity;

        let rate_limit_exceeded = {
            let mut buckets = self.ip_buckets.write();
            let bucket = buckets
                .entry(ctx.ip)
                .or_insert_with(|| TokenBucket::new(capacity, refill_rate));

            !bucket.try_consume(1)
        };

        if rate_limit_exceeded {
            let _ = self
                .emit_event(SecurityEvent::RateLimitExceeded {
                    ip: ctx.ip,
                    endpoint: ctx.endpoint.clone(),
                    current_rate: 1, // We know we tried to consume 1 token
                    limit: requests_per_second,
                })
                .await;

            return Err(SecurityError::RateLimitExceeded {
                reason: format!(
                    "IP rate limit exceeded: {} requests per second",
                    requests_per_second
                ),
            });
        }

        Ok(())
    }

    /// Check DPoP key-based rate limiting
    async fn check_dpop_rate_limit(
        &self,
        _ctx: &SecurityContext,
        thumbprint: &str,
        requests_per_second: u32,
        burst_capacity: u32,
    ) -> SecurityResult<()> {
        let mut buckets = self.dpop_buckets.write();
        let refill_rate = requests_per_second as f64;
        let capacity = burst_capacity;

        let bucket = buckets
            .entry(thumbprint.to_string())
            .or_insert_with(|| TokenBucket::new(capacity, refill_rate));

        if !bucket.try_consume(1) {
            return Err(SecurityError::RateLimitExceeded {
                reason: format!(
                    "DPoP key rate limit exceeded: {} requests per second",
                    requests_per_second
                ),
            });
        }

        Ok(())
    }

    /// Check circuit breakers
    async fn check_circuit_breakers(&self, ctx: &SecurityContext) -> SecurityResult<()> {
        if !self.config.circuit_breaker.enabled {
            return Ok(());
        }

        let mut breakers = self.circuit_breakers.write();
        let breaker = breakers
            .entry(ctx.endpoint.clone())
            .or_insert_with(|| CircuitBreaker::new(self.config.circuit_breaker.clone()));

        if !breaker.call_permitted() {
            return Err(SecurityError::CircuitBreakerOpen {
                reason: format!("Circuit breaker open for endpoint: {}", ctx.endpoint),
            });
        }

        Ok(())
    }

    /// Check DPoP-specific security
    async fn check_dpop_security(&self, _ctx: &SecurityContext) -> SecurityResult<()> {
        // This would integrate with DPoP proof validation
        // For now, we implement basic checks

        if let Some(dpop_config) = &self.config.dpop
            && dpop_config.key_tracking
        {
            // DPoP replay detection would be implemented here
            // This would check against a nonce store
        }

        if let Some(dpop_config) = &self.config.dpop
            && dpop_config.key_rotation_detection
        {
            // Monitor for suspicious key rotation patterns
        }

        Ok(())
    }

    /// Update activity monitoring for DoS protection
    async fn update_activity_monitoring(&self, ctx: &SecurityContext) {
        if !self.config.dos_protection.enabled {
            return;
        }

        let now = Instant::now();
        let (should_block, current_count) = {
            let mut suspicious_ips = self.suspicious_ips.write();

            let (count, first_seen) = suspicious_ips.entry(ctx.ip).or_insert((0, now));

            *count += 1;

            // Check if we're in a fresh time window
            if now.duration_since(*first_seen) > Duration::from_secs(60) {
                *count = 1;
                *first_seen = now;
            }

            (
                *count > self.config.dos_protection.max_requests_per_minute,
                *count,
            )
        };

        // Check thresholds
        if should_block {
            // Block the IP
            {
                let mut blocked_ips = self.blocked_ips.write();
                blocked_ips.insert(
                    ctx.ip,
                    now + Duration::from_secs(self.config.dos_protection.block_duration_seconds),
                );
            }

            let _ = self
                .emit_event(SecurityEvent::IpBlocked {
                    ip: ctx.ip,
                    reason: "Exceeded request threshold".to_string(),
                    duration: Duration::from_secs(
                        self.config.dos_protection.block_duration_seconds,
                    ),
                })
                .await;

            info!(
                ip = %ctx.ip,
                count = current_count,
                "IP blocked due to excessive requests"
            );
        } else if current_count > self.config.dos_protection.suspicious_threshold {
            let severity =
                (current_count * 10 / self.config.dos_protection.max_requests_per_minute) as u8;

            let _ = self
                .emit_event(SecurityEvent::SuspiciousActivity {
                    ip: ctx.ip,
                    pattern: "High request rate".to_string(),
                    severity,
                })
                .await;
        }
    }

    /// Emit security event
    async fn emit_event(
        &self,
        event: SecurityEvent,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<SecurityEvent>> {
        if let Some(ref sender) = self.event_sender {
            sender.send(event)?;
        }
        Ok(())
    }

    /// Cleanup old data (should be called periodically)
    pub async fn cleanup(&self) {
        let now = Instant::now();

        // Cleanup blocked IPs
        {
            let mut blocked_ips = self.blocked_ips.write();
            blocked_ips.retain(|_, &mut blocked_until| blocked_until > now);
        }

        // Cleanup suspicious IPs (keep recent activity)
        {
            let mut suspicious_ips = self.suspicious_ips.write();
            suspicious_ips.retain(|_, (_, first_seen)| {
                now.duration_since(*first_seen) < Duration::from_secs(3600) // Keep 1 hour of history
            });
        }

        // Cleanup old token buckets that haven't been used
        {
            let mut ip_buckets = self.ip_buckets.write();
            ip_buckets.retain(|_, bucket| {
                now.duration_since(bucket.last_refill) < Duration::from_secs(3600)
            });
        }

        {
            let mut dpop_buckets = self.dpop_buckets.write();
            dpop_buckets.retain(|_, bucket| {
                now.duration_since(bucket.last_refill) < Duration::from_secs(3600)
            });
        }

        debug!("Security middleware cleanup completed");
    }

    /// Get current security statistics
    pub fn get_stats(&self) -> SecurityStats {
        let blocked_count = self.blocked_ips.read().len();
        let suspicious_count = self.suspicious_ips.read().len();
        let tracked_ips = self.ip_buckets.read().len();
        let tracked_dpop_keys = self.dpop_buckets.read().len();
        let circuit_breaker_count = self.circuit_breakers.read().len();

        SecurityStats {
            blocked_ip_count: blocked_count,
            suspicious_ip_count: suspicious_count,
            tracked_ip_count: tracked_ips,
            tracked_dpop_key_count: tracked_dpop_keys,
            circuit_breaker_count,
            uptime: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl Middleware for SecurityMiddleware {
    async fn process_request(
        &self,
        request: &mut JsonRpcRequest,
        ctx: &mut RequestContext,
    ) -> crate::ServerResult<()> {
        // Extract security context from request context
        let ip = ctx
            .metadata
            .get("client_ip")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

        // Extract DPoP thumbprint if present
        let dpop_thumbprint = ctx
            .metadata
            .get("dpop_thumbprint")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let security_ctx = SecurityContext {
            ip,
            endpoint: request.method.clone(),
            dpop_thumbprint,
            timestamp: SystemTime::now(),
            request_id: ctx.request_id.to_string(),
        };

        // Check security constraints
        if let Err(security_error) = self.check_request(&security_ctx).await {
            return match security_error {
                SecurityError::RateLimitExceeded { reason } => {
                    Err(ServerError::rate_limit_with_retry(reason, 60))
                }
                SecurityError::IpBlocked { reason } => Err(ServerError::authorization(format!(
                    "Access denied: {}",
                    reason
                ))),
                SecurityError::CircuitBreakerOpen { reason } => {
                    Err(ServerError::resource_exhausted(reason))
                }
                SecurityError::DpopViolation { reason } => Err(ServerError::authentication(
                    format!("DPoP security violation: {}", reason),
                )),
                SecurityError::Configuration { reason } => Err(ServerError::configuration(
                    format!("Security configuration error: {}", reason),
                )),
            };
        }

        // Record successful request for monitoring
        self.record_success(&security_ctx).await;

        Ok(())
    }

    async fn process_response(
        &self,
        _response: &mut JsonRpcResponse,
        _ctx: &RequestContext,
    ) -> crate::ServerResult<()> {
        // Security middleware primarily works on requests
        // Response processing could include adding security headers or metrics
        Ok(())
    }

    fn name(&self) -> &str {
        "security"
    }

    fn priority(&self) -> u32 {
        5 // Very high priority - run early in the middleware stack
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Security statistics
#[derive(Debug, Clone, Serialize)]
pub struct SecurityStats {
    /// Number of currently blocked IPs
    pub blocked_ip_count: usize,
    /// Number of IPs flagged as suspicious
    pub suspicious_ip_count: usize,
    /// Number of IPs being tracked for rate limiting
    pub tracked_ip_count: usize,
    /// Number of DPoP keys being tracked for rate limiting
    pub tracked_dpop_key_count: usize,
    /// Number of active circuit breakers
    pub circuit_breaker_count: usize,
    /// Middleware uptime
    pub uptime: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(config.enabled);
        assert!(config.dos_protection.enabled);
        assert!(config.circuit_breaker.enabled);
        assert!(config.event_logging);

        // Check default rate limiting strategy
        match config.rate_limiting_strategy {
            RateLimitingStrategy::PerIp {
                requests_per_second,
                burst_capacity,
            } => {
                assert!(requests_per_second > 0);
                assert!(burst_capacity > 0);
            }
            _ => panic!("Expected PerIp rate limiting strategy by default"),
        }
    }

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10, 1.0); // 10 capacity, 1 token per second

        // Should allow initial burst
        assert!(bucket.try_consume(5));
        assert_eq!(bucket.tokens, 5);

        // Should reject when over capacity
        assert!(!bucket.try_consume(10));
    }

    #[test]
    fn test_circuit_breaker_states() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3.0,
            recovery_time_seconds: 1,
            ..Default::default()
        };

        let mut breaker = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(breaker.state, CircuitBreakerState::Closed);
        assert!(breaker.call_permitted());

        // Record failures
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_failure();

        // Should be open now
        assert_eq!(breaker.state, CircuitBreakerState::Open);
        assert!(!breaker.call_permitted());
    }

    #[tokio::test]
    async fn test_security_middleware_rate_limiting() {
        let config = SecurityConfig {
            rate_limiting_strategy: RateLimitingStrategy::PerIp {
                requests_per_second: 5,
                burst_capacity: 5,
            },
            ..Default::default()
        };

        let middleware = SecurityMiddleware::new(config);
        let ctx = SecurityContext {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            endpoint: "/test".to_string(),
            dpop_thumbprint: None,
            timestamp: SystemTime::now(),
            request_id: "test-123".to_string(),
        };

        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(middleware.check_request(&ctx).await.is_ok());
        }

        // 6th request should be rate limited
        assert!(middleware.check_request(&ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_ip_blocking() {
        let config = SecurityConfig {
            dos_protection: crate::config::DoSProtectionConfig {
                enabled: true,
                suspicious_threshold: 2,
                max_requests_per_minute: 3,
                block_duration_seconds: 60,
            },
            ..Default::default()
        };

        let middleware = SecurityMiddleware::new(config);
        let ctx = SecurityContext {
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            endpoint: "/api/test".to_string(),
            dpop_thumbprint: None,
            timestamp: SystemTime::now(),
            request_id: "test-456".to_string(),
        };

        // Simulate activity that should trigger blocking
        for _ in 0..4 {
            middleware.update_activity_monitoring(&ctx).await;
        }

        // IP should now be blocked
        assert!(middleware.check_ip_blocked(&ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let middleware = SecurityMiddleware::new(SecurityConfig::default());

        // Add some test data
        {
            let mut blocked_ips = middleware.blocked_ips.write();
            blocked_ips.insert(
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                Instant::now() - Duration::from_secs(7200), // 2 hours ago (expired)
            );
            blocked_ips.insert(
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
                Instant::now() + Duration::from_secs(3600), // 1 hour from now (active)
            );
        }

        middleware.cleanup().await;

        // Should have cleaned up expired entries
        let blocked_ips = middleware.blocked_ips.read();
        assert_eq!(blocked_ips.len(), 1); // Only the active block should remain
    }

    #[test]
    fn test_security_stats() {
        let middleware = SecurityMiddleware::new(SecurityConfig::default());
        let stats = middleware.get_stats();

        assert_eq!(stats.blocked_ip_count, 0);
        assert_eq!(stats.suspicious_ip_count, 0);
        assert_eq!(stats.tracked_ip_count, 0);
        assert_eq!(stats.circuit_breaker_count, 0);
    }
}
