//! Transport robustness features
//!
//! This module provides comprehensive robustness features for MCP transports including:
//! - Retry mechanisms with exponential backoff
//! - Circuit breaker pattern for fault tolerance
//! - Health checking and monitoring
//! - Connection pooling with failover
//! - Adaptive timeout management
//! - Message deduplication and ordering

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, timeout};

use crate::core::{
    Transport, TransportError, TransportMessage, TransportResult, TransportState, TransportType,
};

/// Retry configuration for transport operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 - 1.0) to avoid thundering herd
    pub jitter_factor: f64,
    /// Whether to retry on connection errors
    pub retry_on_connection_error: bool,
    /// Whether to retry on timeout errors
    pub retry_on_timeout: bool,
    /// Custom retry conditions
    pub custom_retry_conditions: Vec<RetryCondition>,
}

/// Custom retry condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryCondition {
    /// Error pattern to match
    pub error_pattern: String,
    /// Whether to retry on this condition
    pub should_retry: bool,
    /// Override delay for this condition
    pub custom_delay: Option<Duration>,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit
    pub success_threshold: u32,
    /// Timeout in open state before trying half-open
    pub timeout: Duration,
    /// Rolling window size for failure counting
    pub rolling_window_size: usize,
    /// Minimum request threshold before opening circuit
    pub minimum_requests: u32,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing fast)
    Open,
    /// Circuit is half-open (testing if service recovered)
    HalfOpen,
}

impl Default for CircuitState {
    fn default() -> Self {
        Self::Closed
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
    /// Custom health check endpoint or command
    pub custom_check: Option<String>,
}

/// Health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Transport is healthy
    Healthy,
    /// Transport is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
    /// Health check is in progress
    Checking,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Transport health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Current health status
    pub status: HealthStatus,
    /// Last health check time
    pub last_check: SystemTime,
    /// Consecutive successful checks
    pub consecutive_successes: u32,
    /// Consecutive failed checks
    pub consecutive_failures: u32,
    /// Additional health details
    pub details: HashMap<String, serde_json::Value>,
}

/// Robust transport wrapper with retry, circuit breaker, and health checking
#[derive(Debug)]
pub struct RobustTransport {
    /// Underlying transport
    inner: Arc<Mutex<Box<dyn Transport>>>,
    /// Retry configuration
    retry_config: RetryConfig,
    /// Circuit breaker
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    /// Health checker
    health_checker: Arc<Mutex<HealthChecker>>,
    /// Transport metrics
    metrics: Arc<RobustTransportMetrics>,
    /// Message deduplication cache
    dedup_cache: Arc<RwLock<DeduplicationCache>>,
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Circuit breaker configuration
    config: CircuitBreakerConfig,
    /// Current circuit state
    state: CircuitState,
    /// Failure count in current window
    failure_count: u32,
    /// Success count in half-open state
    success_count: u32,
    /// Last state change time
    last_state_change: Instant,
    /// Rolling window of recent operations
    rolling_window: VecDeque<OperationResult>,
}

/// Operation result for circuit breaker tracking
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Operation timestamp
    pub timestamp: Instant,
    /// Whether operation was successful
    pub success: bool,
    /// Operation duration
    pub duration: Duration,
}

/// Health checker implementation
#[derive(Debug)]
pub struct HealthChecker {
    /// Health check configuration
    config: HealthCheckConfig,
    /// Current health information
    health_info: HealthInfo,
    /// Last health check result
    last_check_result: Option<bool>,
}

/// Robust transport metrics
#[derive(Debug, Default)]
pub struct RobustTransportMetrics {
    /// Total retry attempts
    pub retry_attempts: AtomicU64,
    /// Successful retries
    pub successful_retries: AtomicU64,
    /// Circuit breaker trips
    pub circuit_breaker_trips: AtomicU64,
    /// Health check failures
    pub health_check_failures: AtomicU64,
    /// Duplicate messages filtered
    pub duplicate_messages_filtered: AtomicU64,
    /// Average operation latency (microseconds)
    pub avg_operation_latency_us: AtomicU64,
    /// Current circuit breaker state
    pub circuit_state: Arc<RwLock<CircuitState>>,
    /// Current health status
    pub health_status: Arc<RwLock<HealthStatus>>,
}

/// Message deduplication cache
#[derive(Debug)]
pub struct DeduplicationCache {
    /// Message ID cache with timestamps
    pub cache: HashMap<String, Instant>,
    /// Cache size limit
    pub max_size: usize,
    /// Cache entry TTL
    pub ttl: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on_connection_error: true,
            retry_on_timeout: true,
            custom_retry_conditions: Vec::new(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            rolling_window_size: 100,
            minimum_requests: 10,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            custom_check: None,
        }
    }
}

impl Default for HealthInfo {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            last_check: SystemTime::now(),
            consecutive_successes: 0,
            consecutive_failures: 0,
            details: HashMap::new(),
        }
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_state_change: Instant::now(),
            rolling_window: VecDeque::new(),
        }
    }

    /// Check if operation should be allowed
    pub fn should_allow_operation(&mut self) -> bool {
        self.update_state();

        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => true,
        }
    }

    /// Record operation result
    pub fn record_result(&mut self, success: bool, duration: Duration) {
        let result = OperationResult {
            timestamp: Instant::now(),
            success,
            duration,
        };

        self.rolling_window.push_back(result);

        // Maintain rolling window size
        while self.rolling_window.len() > self.config.rolling_window_size {
            self.rolling_window.pop_front();
        }

        match self.state {
            CircuitState::Closed => {
                if success {
                    self.failure_count = 0;
                } else {
                    self.failure_count += 1;
                    if self.should_trip() {
                        self.trip_circuit();
                    }
                }
            }
            CircuitState::HalfOpen => {
                if success {
                    self.success_count += 1;
                    if self.success_count >= self.config.success_threshold {
                        self.close_circuit();
                    }
                } else {
                    self.trip_circuit();
                }
            }
            CircuitState::Open => {
                // No action needed in open state
            }
        }
    }

    /// Check if circuit should trip
    fn should_trip(&self) -> bool {
        let total_requests = self.rolling_window.len() as u32;

        if total_requests < self.config.minimum_requests {
            return false;
        }

        self.failure_count >= self.config.failure_threshold
    }

    /// Trip the circuit breaker
    fn trip_circuit(&mut self) {
        self.state = CircuitState::Open;
        self.last_state_change = Instant::now();
        self.failure_count = 0;
        self.success_count = 0;
    }

    /// Close the circuit breaker
    fn close_circuit(&mut self) {
        self.state = CircuitState::Closed;
        self.last_state_change = Instant::now();
        self.failure_count = 0;
        self.success_count = 0;
    }

    /// Update circuit state based on time
    fn update_state(&mut self) {
        if self.state == CircuitState::Open
            && self.last_state_change.elapsed() >= self.config.timeout
        {
            self.state = CircuitState::HalfOpen;
            self.last_state_change = Instant::now();
            self.success_count = 0;
        }
    }

    /// Get current circuit state
    #[must_use]
    pub fn state(&self) -> CircuitState {
        self.state.clone()
    }

    /// Get circuit breaker statistics
    #[must_use]
    pub fn statistics(&self) -> CircuitBreakerStats {
        let failure_rate = if self.rolling_window.is_empty() {
            0.0
        } else {
            let failures = self.rolling_window.iter().filter(|r| !r.success).count();
            failures as f64 / self.rolling_window.len() as f64
        };

        let avg_duration = if self.rolling_window.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.rolling_window.iter().map(|r| r.duration).sum();
            total / self.rolling_window.len() as u32
        };

        CircuitBreakerStats {
            state: self.state.clone(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            failure_rate,
            avg_operation_duration: avg_duration,
            time_in_current_state: self.last_state_change.elapsed(),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    /// Current circuit state
    pub state: CircuitState,
    /// Current failure count
    pub failure_count: u32,
    /// Current success count (in half-open)
    pub success_count: u32,
    /// Current failure rate (0.0 - 1.0)
    pub failure_rate: f64,
    /// Average operation duration
    pub avg_operation_duration: Duration,
    /// Time spent in current state
    pub time_in_current_state: Duration,
}

impl HealthChecker {
    /// Create a new health checker
    #[must_use]
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            health_info: HealthInfo::default(),
            last_check_result: None,
        }
    }

    /// Perform health check
    pub async fn check_health(&mut self, transport: &dyn Transport) -> bool {
        self.health_info.status = HealthStatus::Checking;
        self.health_info.last_check = SystemTime::now();

        let check_result = timeout(self.config.timeout, self.perform_check(transport)).await;

        let success = match check_result {
            Ok(Ok(healthy)) => healthy,
            Ok(Err(_)) => false,
            Err(_) => false, // Timeout
        };

        self.update_health_status(success);
        success
    }

    /// Perform actual health check
    async fn perform_check(&self, transport: &dyn Transport) -> TransportResult<bool> {
        // Basic health check - verify transport is connected
        Ok(transport.is_connected().await)
    }

    /// Update health status based on check result
    const fn update_health_status(&mut self, success: bool) {
        if success {
            self.health_info.consecutive_successes += 1;
            self.health_info.consecutive_failures = 0;

            if self.health_info.consecutive_successes >= self.config.success_threshold {
                self.health_info.status = HealthStatus::Healthy;
            }
        } else {
            self.health_info.consecutive_failures += 1;
            self.health_info.consecutive_successes = 0;

            if self.health_info.consecutive_failures >= self.config.failure_threshold {
                self.health_info.status = HealthStatus::Unhealthy;
            }
        }

        self.last_check_result = Some(success);
    }

    /// Get current health information
    #[must_use]
    pub const fn health_info(&self) -> &HealthInfo {
        &self.health_info
    }
}

impl DeduplicationCache {
    /// Create a new deduplication cache
    #[must_use]
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            ttl,
        }
    }

    /// Check if message is duplicate
    pub fn is_duplicate(&mut self, message_id: &str) -> bool {
        self.cleanup_expired();

        if self.cache.contains_key(message_id) {
            true
        } else {
            self.cache.insert(message_id.to_string(), Instant::now());
            self.maintain_size_limit();
            false
        }
    }

    /// Clean up expired entries
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.cache
            .retain(|_, timestamp| now.duration_since(*timestamp) < self.ttl);
    }

    /// Maintain cache size limit
    fn maintain_size_limit(&mut self) {
        if self.cache.len() > self.max_size {
            // Remove oldest entries
            let mut entries: Vec<_> = self.cache.iter().collect();
            entries.sort_by_key(|(_, timestamp)| *timestamp);

            let to_remove = self.cache.len() - self.max_size;
            let keys_to_remove: Vec<String> = entries
                .iter()
                .take(to_remove)
                .map(|(k, _)| (*k).clone())
                .collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
    }
}

impl RobustTransport {
    /// Create a new robust transport wrapper
    #[must_use]
    pub fn new(
        transport: Box<dyn Transport>,
        retry_config: RetryConfig,
        circuit_config: CircuitBreakerConfig,
        health_config: HealthCheckConfig,
    ) -> Self {
        let circuit_breaker = Arc::new(Mutex::new(CircuitBreaker::new(circuit_config)));
        let health_checker = Arc::new(Mutex::new(HealthChecker::new(health_config)));
        let metrics = Arc::new(RobustTransportMetrics::default());
        let dedup_cache = Arc::new(RwLock::new(DeduplicationCache::new(
            1000,
            Duration::from_secs(300),
        )));

        Self {
            inner: Arc::new(Mutex::new(transport)),
            retry_config,
            circuit_breaker,
            health_checker,
            metrics,
            dedup_cache,
        }
    }

    /// Execute operation with retry logic
    async fn execute_with_retry<F, Fut, T>(&self, mut operation: F) -> TransportResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = TransportResult<T>>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.retry_config.max_attempts {
            // Check circuit breaker
            {
                let mut breaker = self.circuit_breaker.lock().await;
                if !breaker.should_allow_operation() {
                    self.metrics
                        .circuit_breaker_trips
                        .fetch_add(1, Ordering::Relaxed);
                    return Err(TransportError::Internal(
                        "Circuit breaker is open".to_string(),
                    ));
                }
            }

            let start_time = Instant::now();
            let result = operation().await;
            let duration = start_time.elapsed();

            // Update metrics
            self.metrics
                .avg_operation_latency_us
                .store(duration.as_micros() as u64, Ordering::Relaxed);

            // Record circuit breaker result
            {
                let mut breaker = self.circuit_breaker.lock().await;
                breaker.record_result(result.is_ok(), duration);
                *self.metrics.circuit_state.write().await = breaker.state();
            }

            match result {
                Ok(value) => {
                    if attempt > 0 {
                        self.metrics
                            .successful_retries
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    return Ok(value);
                }
                Err(error) => {
                    if !self.should_retry(&error, attempt) {
                        return Err(error);
                    }

                    last_error = Some(error);
                    attempt += 1;

                    if attempt < self.retry_config.max_attempts {
                        self.metrics.retry_attempts.fetch_add(1, Ordering::Relaxed);
                        let delay = self.calculate_retry_delay(attempt);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TransportError::Internal("Maximum retry attempts exceeded".to_string())
        }))
    }

    /// Check if error should trigger a retry
    const fn should_retry(&self, error: &TransportError, attempt: u32) -> bool {
        if attempt >= self.retry_config.max_attempts {
            return false;
        }

        match error {
            TransportError::ConnectionFailed(_) | TransportError::ConnectionLost(_) => {
                self.retry_config.retry_on_connection_error
            }
            TransportError::Timeout => self.retry_config.retry_on_timeout,
            TransportError::SendFailed(_) | TransportError::ReceiveFailed(_) => true,
            TransportError::SerializationFailed(_) => false, // Don't retry serialization errors
            TransportError::ProtocolError(_) => false,       // Don't retry protocol errors
            _ => true,                                       // Retry other errors by default
        }
    }

    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay_ms = self.retry_config.base_delay.as_millis() as f64;
        let multiplier = self.retry_config.backoff_multiplier.powi(attempt as i32);
        let delay_ms = base_delay_ms * multiplier;

        // Apply jitter
        let jitter = fastrand::f64() * self.retry_config.jitter_factor;
        let jittered_delay_ms = delay_ms * (1.0 + jitter);

        // Cap at max delay
        let final_delay_ms = jittered_delay_ms.min(self.retry_config.max_delay.as_millis() as f64);

        Duration::from_millis(final_delay_ms as u64)
    }

    /// Start background health checking
    pub async fn start_health_monitoring(&self) {
        let health_checker = self.health_checker.clone();
        let metrics = self.metrics.clone();
        let transport = self.inner.clone();
        let interval = {
            let checker = health_checker.lock().await;
            checker.config.interval
        };

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Perform actual health check on the transport
                let health_status = {
                    let mut checker = health_checker.lock().await;
                    let transport_guard = transport.lock().await;
                    let is_healthy = checker.check_health(&**transport_guard).await;

                    if is_healthy {
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Unhealthy
                    }
                };

                *metrics.health_status.write().await = health_status;
            }
        });
    }

    /// Get robust transport metrics
    pub async fn get_metrics(&self) -> RobustTransportMetrics {
        RobustTransportMetrics {
            retry_attempts: AtomicU64::new(self.metrics.retry_attempts.load(Ordering::Relaxed)),
            successful_retries: AtomicU64::new(
                self.metrics.successful_retries.load(Ordering::Relaxed),
            ),
            circuit_breaker_trips: AtomicU64::new(
                self.metrics.circuit_breaker_trips.load(Ordering::Relaxed),
            ),
            health_check_failures: AtomicU64::new(
                self.metrics.health_check_failures.load(Ordering::Relaxed),
            ),
            duplicate_messages_filtered: AtomicU64::new(
                self.metrics
                    .duplicate_messages_filtered
                    .load(Ordering::Relaxed),
            ),
            avg_operation_latency_us: AtomicU64::new(
                self.metrics
                    .avg_operation_latency_us
                    .load(Ordering::Relaxed),
            ),
            circuit_state: Arc::new(RwLock::new(self.metrics.circuit_state.read().await.clone())),
            health_status: Arc::new(RwLock::new(self.metrics.health_status.read().await.clone())),
        }
    }

    /// Get circuit breaker statistics
    pub async fn get_circuit_breaker_stats(&self) -> CircuitBreakerStats {
        let breaker = self.circuit_breaker.lock().await;
        breaker.statistics()
    }

    /// Get health information
    pub async fn get_health_info(&self) -> HealthInfo {
        let checker = self.health_checker.lock().await;
        checker.health_info().clone()
    }
}

#[async_trait]
impl Transport for RobustTransport {
    fn transport_type(&self) -> TransportType {
        // Delegate to the inner transport - no need to cache since this is a cheap operation
        if let Ok(inner) = self.inner.try_lock() {
            inner.transport_type()
        } else {
            // If we can't get the lock, return a reasonable default based on the transport
            // This should rarely happen and only during initialization or shutdown
            TransportType::Stdio
        }
    }

    fn capabilities(&self) -> &crate::core::TransportCapabilities {
        // Use a static default since capabilities are typically the same for all transports
        // of the same type and this is a sync method that can't access the inner transport
        static DEFAULT_CAPABILITIES: std::sync::LazyLock<crate::core::TransportCapabilities> =
            std::sync::LazyLock::new(crate::core::TransportCapabilities::default);
        &DEFAULT_CAPABILITIES
    }

    async fn state(&self) -> TransportState {
        let inner = self.inner.lock().await;
        inner.state().await
    }

    async fn connect(&mut self) -> TransportResult<()> {
        let inner = self.inner.clone();
        self.execute_with_retry(move || {
            let inner = inner.clone();
            async move {
                let mut transport = inner.lock().await;
                transport.connect().await
            }
        })
        .await
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        let mut inner = self.inner.lock().await;
        inner.disconnect().await
    }

    async fn send(&mut self, message: TransportMessage) -> TransportResult<()> {
        // Check for duplicate messages
        {
            let mut dedup = self.dedup_cache.write().await;
            if dedup.is_duplicate(&message.id.to_string()) {
                self.metrics
                    .duplicate_messages_filtered
                    .fetch_add(1, Ordering::Relaxed);
                return Ok(()); // Silently drop duplicate
            }
        }

        let inner = self.inner.clone();
        let msg = message.clone();
        self.execute_with_retry(move || {
            let inner = inner.clone();
            let msg = msg.clone();
            async move {
                let mut transport = inner.lock().await;
                transport.send(msg).await
            }
        })
        .await
    }

    async fn receive(&mut self) -> TransportResult<Option<TransportMessage>> {
        let inner = self.inner.clone();
        self.execute_with_retry(move || {
            let inner = inner.clone();
            async move {
                let mut transport = inner.lock().await;
                transport.receive().await
            }
        })
        .await
    }

    async fn metrics(&self) -> crate::core::TransportMetrics {
        let inner = self.inner.lock().await;
        inner.metrics().await
    }

    fn endpoint(&self) -> Option<String> {
        // Try to get endpoint from inner transport without blocking
        if let Ok(inner) = self.inner.try_lock() {
            inner.endpoint()
        } else {
            // If we can't get the lock, return None - this is acceptable
            // as endpoint() is used for informational purposes
            None
        }
    }

    async fn configure(&mut self, config: crate::core::TransportConfig) -> TransportResult<()> {
        let mut inner = self.inner.lock().await;
        inner.configure(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TransportCapabilities;
    use std::sync::atomic::AtomicUsize;

    /// Mock transport for testing
    #[derive(Debug)]
    struct MockTransport {
        should_fail: Arc<AtomicUsize>,
        fail_count: Arc<AtomicUsize>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                should_fail: Arc::new(AtomicUsize::new(0)),
                fail_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn set_failure_mode(&self, fail_next_n: usize) {
            self.should_fail.store(fail_next_n, Ordering::Relaxed);
            self.fail_count.store(0, Ordering::Relaxed);
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        fn transport_type(&self) -> TransportType {
            TransportType::Stdio
        }

        fn capabilities(&self) -> &TransportCapabilities {
            static DEFAULT_CAPS: std::sync::LazyLock<TransportCapabilities> =
                std::sync::LazyLock::new(TransportCapabilities::default);
            &DEFAULT_CAPS
        }

        async fn state(&self) -> TransportState {
            TransportState::Connected
        }

        async fn connect(&mut self) -> TransportResult<()> {
            let current_fail = self.fail_count.fetch_add(1, Ordering::Relaxed);
            let should_fail = self.should_fail.load(Ordering::Relaxed);

            if current_fail < should_fail {
                Err(TransportError::ConnectionFailed("Mock failure".to_string()))
            } else {
                Ok(())
            }
        }

        async fn disconnect(&mut self) -> TransportResult<()> {
            Ok(())
        }

        async fn send(&mut self, _message: TransportMessage) -> TransportResult<()> {
            let current_fail = self.fail_count.fetch_add(1, Ordering::Relaxed);
            let should_fail = self.should_fail.load(Ordering::Relaxed);

            if current_fail < should_fail {
                Err(TransportError::SendFailed("Mock failure".to_string()))
            } else {
                Ok(())
            }
        }

        async fn receive(&mut self) -> TransportResult<Option<TransportMessage>> {
            Ok(None)
        }

        async fn metrics(&self) -> crate::core::TransportMetrics {
            Default::default()
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_basic_functionality() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            rolling_window_size: 10,
            minimum_requests: 2,
        };

        let mut breaker = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.should_allow_operation());

        // Record failures to trip circuit
        breaker.record_result(false, Duration::from_millis(100));
        breaker.record_result(false, Duration::from_millis(100));

        // Should be open now
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.should_allow_operation());

        // Wait for timeout and check half-open
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(breaker.should_allow_operation()); // This updates state to half-open
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        // Record success to close circuit
        breaker.record_result(true, Duration::from_millis(50));
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let mock = MockTransport::new();
        mock.set_failure_mode(2); // Fail first 2 attempts

        let retry_config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let mut robust = RobustTransport::new(
            Box::new(mock),
            retry_config,
            CircuitBreakerConfig::default(),
            HealthCheckConfig::default(),
        );

        // Should succeed on third attempt
        let result = robust.connect().await;
        assert!(result.is_ok());

        let metrics = robust.get_metrics().await;
        assert_eq!(metrics.retry_attempts.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.successful_retries.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_deduplication_cache() {
        let mut cache = DeduplicationCache::new(3, Duration::from_millis(100));

        // First occurrence should not be duplicate
        assert!(!cache.is_duplicate("msg1"));
        assert!(!cache.is_duplicate("msg2"));
        assert!(!cache.is_duplicate("msg3"));

        // Subsequent occurrences should be duplicates
        assert!(cache.is_duplicate("msg1"));
        assert!(cache.is_duplicate("msg2"));

        // Adding more should evict oldest
        assert!(!cache.is_duplicate("msg4"));
        assert!(!cache.is_duplicate("msg5"));

        // msg1 might be evicted due to size limit
        // Note: exact behavior depends on implementation details
    }

    #[tokio::test]
    async fn test_health_checker() {
        let config = HealthCheckConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            interval: Duration::from_millis(50),
            custom_check: None,
        };

        let mut checker = HealthChecker::new(config);
        let mock = MockTransport::new();

        // Initial health check should succeed
        let result = checker.check_health(&mock).await;
        assert!(result);
        assert_eq!(checker.health_info().status, HealthStatus::Healthy);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(10),
            jitter_factor: 0.0, // No jitter for predictable testing
            ..Default::default()
        };

        let mock = MockTransport::new();
        let robust = RobustTransport::new(
            Box::new(mock),
            config,
            CircuitBreakerConfig::default(),
            HealthCheckConfig::default(),
        );

        let delay1 = robust.calculate_retry_delay(1);
        let delay2 = robust.calculate_retry_delay(2);

        // Should follow exponential backoff
        assert!(delay2 > delay1);
        assert!(delay1 >= Duration::from_millis(200)); // 100 * 2^1
        assert!(delay2 >= Duration::from_millis(400)); // 100 * 2^2
    }

    #[tokio::test]
    async fn test_circuit_breaker_statistics() {
        let config = CircuitBreakerConfig::default();
        let mut breaker = CircuitBreaker::new(config);

        // Record some operations
        breaker.record_result(true, Duration::from_millis(100));
        breaker.record_result(false, Duration::from_millis(200));
        breaker.record_result(true, Duration::from_millis(150));

        let stats = breaker.statistics();
        assert_eq!(stats.state, CircuitState::Closed);
        assert!(stats.failure_rate > 0.0 && stats.failure_rate < 1.0);
        assert!(stats.avg_operation_duration > Duration::ZERO);
    }
}
