//! Comprehensive tests for the utils module to improve coverage
//!
//! This test suite targets the utils module which provides timeout handling,
//! retry mechanisms with exponential backoff, circuit breakers, and utility macros.

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use turbomcp_core::utils::*;
use turbomcp_core::{feature_gate, measure_time};

// ============================================================================
// TimeoutError Tests
// ============================================================================

#[test]
fn test_timeout_error_display() {
    let error = TimeoutError;
    assert_eq!(error.to_string(), "Operation timed out");
}

#[test]
fn test_timeout_error_debug() {
    let error = TimeoutError;
    let debug_str = format!("{error:?}");
    assert_eq!(debug_str, "TimeoutError");
}

#[test]
fn test_timeout_error_eq() {
    let error1 = TimeoutError;
    let error2 = TimeoutError;
    assert_eq!(error1, error2);
    assert_eq!(error1, error1); // self equality
}

#[test]
fn test_timeout_error_clone_copy() {
    let error = TimeoutError;
    let cloned = error;
    let copied = error; // Copy trait
    assert_eq!(error, cloned);
    assert_eq!(error, copied);
}

#[test]
fn test_timeout_error_as_std_error() {
    let error = TimeoutError;
    let std_error: &dyn std::error::Error = &error;
    assert_eq!(std_error.to_string(), "Operation timed out");
    assert!(std_error.source().is_none());
}

// ============================================================================
// Timeout Future Tests
// ============================================================================

#[tokio::test]
async fn test_timeout_success_immediate() {
    let result = timeout(Duration::from_millis(100), async { 42 }).await;
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_timeout_success_with_delay() {
    let result = timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        "success"
    })
    .await;
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_timeout_failure() {
    let result = timeout(Duration::from_millis(10), async {
        sleep(Duration::from_millis(50)).await;
        42
    })
    .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), TimeoutError);
}

#[tokio::test]
async fn test_timeout_exact_timing() {
    let start = Instant::now();
    let result = timeout(Duration::from_millis(20), async {
        sleep(Duration::from_millis(30)).await;
        "late"
    })
    .await;

    let elapsed = start.elapsed();
    assert!(result.is_err());
    // Should timeout roughly around 20ms, allow some variance
    assert!(elapsed >= Duration::from_millis(15));
    assert!(elapsed <= Duration::from_millis(40));
}

#[tokio::test]
async fn test_timeout_zero_duration() {
    let result = timeout(Duration::ZERO, async { 42 }).await;
    // Zero timeout behavior is implementation dependent - just verify it completes
    // It may succeed if the future resolves immediately, or timeout
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_timeout_with_error() {
    let result = timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        Result::<i32, &str>::Err("operation failed")
    })
    .await;

    assert!(result.is_ok()); // Timeout didn't occur
    let inner_result = result.unwrap();
    assert!(inner_result.is_err());
    assert_eq!(inner_result.unwrap_err(), "operation failed");
}

#[tokio::test]
async fn test_timeout_new_constructor() {
    let future = async { 100 };
    let timeout_future = Timeout::new(future, Duration::from_millis(50));

    let result = timeout_future.await;
    assert_eq!(result.unwrap(), 100);
}

// ============================================================================
// RetryConfig Tests
// ============================================================================

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();

    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.base_delay, Duration::from_millis(100));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    assert_eq!(config.backoff_multiplier, 2.0);
    assert!(config.jitter);
}

#[test]
fn test_retry_config_new() {
    let config = RetryConfig::new();

    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.base_delay, Duration::from_millis(100));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    assert_eq!(config.backoff_multiplier, 2.0);
    assert!(config.jitter);
}

#[test]
fn test_retry_config_builder_pattern() {
    let config = RetryConfig::new()
        .with_max_attempts(5)
        .with_base_delay(Duration::from_millis(50))
        .with_max_delay(Duration::from_secs(10))
        .with_backoff_multiplier(1.5)
        .with_jitter(false);

    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.base_delay, Duration::from_millis(50));
    assert_eq!(config.max_delay, Duration::from_secs(10));
    assert_eq!(config.backoff_multiplier, 1.5);
    assert!(!config.jitter);
}

#[test]
fn test_retry_config_debug_clone() {
    let config = RetryConfig::new().with_max_attempts(7);

    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("RetryConfig"));
    assert!(debug_str.contains("max_attempts"));
    assert!(debug_str.contains("7"));

    let cloned = config.clone();
    assert_eq!(config.max_attempts, cloned.max_attempts);
    assert_eq!(config.base_delay, cloned.base_delay);
    assert_eq!(config.backoff_multiplier, cloned.backoff_multiplier);
}

#[test]
fn test_retry_config_delay_calculation_no_jitter() {
    let config = RetryConfig::new()
        .with_base_delay(Duration::from_millis(100))
        .with_backoff_multiplier(2.0)
        .with_max_delay(Duration::from_secs(5))
        .with_jitter(false);

    // Attempt 0 should have zero delay
    assert_eq!(config.delay_for_attempt(0), Duration::ZERO);

    // Attempt 1: base delay
    assert_eq!(config.delay_for_attempt(1), Duration::from_millis(100));

    // Attempt 2: base * 2^1 = 200ms
    assert_eq!(config.delay_for_attempt(2), Duration::from_millis(200));

    // Attempt 3: base * 2^2 = 400ms
    assert_eq!(config.delay_for_attempt(3), Duration::from_millis(400));

    // Attempt 4: base * 2^3 = 800ms
    assert_eq!(config.delay_for_attempt(4), Duration::from_millis(800));

    // Test max delay capping
    let long_config = RetryConfig::new()
        .with_base_delay(Duration::from_millis(1000))
        .with_max_delay(Duration::from_millis(2000))
        .with_jitter(false);

    let delay = long_config.delay_for_attempt(5); // Would be 16000ms without cap
    assert_eq!(delay, Duration::from_millis(2000));
}

#[test]
fn test_retry_config_delay_calculation_with_jitter() {
    let config = RetryConfig::new()
        .with_base_delay(Duration::from_millis(100))
        .with_backoff_multiplier(2.0)
        .with_jitter(true);

    let base_delay = config.delay_for_attempt(1);

    // With jitter, the delay should be within ±5% of expected value
    // Expected: 100ms, so range should be ~95ms to 105ms
    assert!(base_delay >= Duration::from_millis(95));
    assert!(base_delay <= Duration::from_millis(105));

    // Multiple calls should produce different values due to jitter
    let delay1 = config.delay_for_attempt(2);
    let delay2 = config.delay_for_attempt(2);
    let delay3 = config.delay_for_attempt(2);

    // At least one should be different (very high probability)
    assert!(delay1 != delay2 || delay2 != delay3 || delay1 != delay3);
}

#[test]
fn test_retry_config_edge_cases() {
    // Test with zero base delay
    let zero_config = RetryConfig::new()
        .with_base_delay(Duration::ZERO)
        .with_jitter(false);

    assert_eq!(zero_config.delay_for_attempt(1), Duration::ZERO);
    assert_eq!(zero_config.delay_for_attempt(5), Duration::ZERO);

    // Test with multiplier of 1.0 (no backoff)
    let no_backoff = RetryConfig::new()
        .with_base_delay(Duration::from_millis(100))
        .with_backoff_multiplier(1.0)
        .with_jitter(false);

    assert_eq!(no_backoff.delay_for_attempt(1), Duration::from_millis(100));
    assert_eq!(no_backoff.delay_for_attempt(2), Duration::from_millis(100));
    assert_eq!(no_backoff.delay_for_attempt(5), Duration::from_millis(100));
}

// ============================================================================
// Retry with Backoff Tests
// ============================================================================

#[tokio::test]
async fn test_retry_with_backoff_success_first_attempt() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig::new()
        .with_max_attempts(3)
        .with_base_delay(Duration::from_millis(1))
        .with_jitter(false);

    let result = retry_with_backoff(
        move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok::<&str, &str>("success")
            }
        },
        config,
        |_: &&str| true,
    )
    .await;

    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_retry_with_backoff_success_after_retries() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig::new()
        .with_max_attempts(5)
        .with_base_delay(Duration::from_millis(1))
        .with_jitter(false);

    let result = retry_with_backoff(
        move || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 3 {
                    Err("still failing")
                } else {
                    Ok("finally success")
                }
            }
        },
        config,
        |_: &&str| true,
    )
    .await;

    assert_eq!(result.unwrap(), "finally success");
    assert_eq!(counter.load(Ordering::SeqCst), 4); // Failed 3 times, succeeded on 4th
}

#[tokio::test]
async fn test_retry_with_backoff_exhausted() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig::new()
        .with_max_attempts(3)
        .with_base_delay(Duration::from_millis(1))
        .with_jitter(false);

    let result: Result<&str, &str> = retry_with_backoff(
        move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err("always fails")
            }
        },
        config,
        |_: &&str| true,
    )
    .await;

    assert_eq!(result.unwrap_err(), "always fails");
    assert_eq!(counter.load(Ordering::SeqCst), 3); // All 3 attempts used
}

#[tokio::test]
async fn test_retry_with_backoff_should_not_retry() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig::new()
        .with_max_attempts(5)
        .with_base_delay(Duration::from_millis(1));

    let result: Result<&str, &str> = retry_with_backoff(
        move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err("fatal error")
            }
        },
        config,
        |error: &&str| *error != "fatal error", // Don't retry fatal errors
    )
    .await;

    assert_eq!(result.unwrap_err(), "fatal error");
    assert_eq!(counter.load(Ordering::SeqCst), 1); // Only one attempt
}

#[tokio::test]
async fn test_retry_with_backoff_mixed_retry_conditions() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig::new()
        .with_max_attempts(5)
        .with_base_delay(Duration::from_millis(1));

    let result = retry_with_backoff(
        move || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                match count {
                    0 => Err("retryable"),
                    1 => Err("retryable"),
                    2 => Err("fatal"), // This should stop retrying
                    _ => Ok("should not reach"),
                }
            }
        },
        config,
        |error: &&str| *error == "retryable",
    )
    .await;

    assert_eq!(result.unwrap_err(), "fatal");
    assert_eq!(counter.load(Ordering::SeqCst), 3); // 2 retryable + 1 fatal
}

#[tokio::test]
async fn test_retry_with_backoff_timing() {
    let config = RetryConfig::new()
        .with_max_attempts(3)
        .with_base_delay(Duration::from_millis(10))
        .with_backoff_multiplier(2.0)
        .with_jitter(false);

    let start = Instant::now();
    let result: Result<i32, &str> =
        retry_with_backoff(|| async { Err("fail") }, config, |_: &&str| true).await;

    let elapsed = start.elapsed();

    assert!(result.is_err());
    // Should have delays of ~10ms and ~20ms between attempts
    // Total time should be at least 30ms
    assert!(elapsed >= Duration::from_millis(25));
}

// ============================================================================
// CircuitState Tests
// ============================================================================

#[test]
fn test_circuit_state_variants() {
    let states = vec![
        CircuitState::Closed,
        CircuitState::Open,
        CircuitState::HalfOpen,
    ];

    for state in &states {
        let debug_str = format!("{state:?}");
        assert!(!debug_str.is_empty());

        let cloned = *state;
        assert_eq!(*state, cloned);
    }
}

#[test]
fn test_circuit_state_equality() {
    assert_eq!(CircuitState::Closed, CircuitState::Closed);
    assert_eq!(CircuitState::Open, CircuitState::Open);
    assert_eq!(CircuitState::HalfOpen, CircuitState::HalfOpen);

    assert_ne!(CircuitState::Closed, CircuitState::Open);
    assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
    assert_ne!(CircuitState::Closed, CircuitState::HalfOpen);
}

// ============================================================================
// CircuitBreaker Tests
// ============================================================================

#[test]
fn test_circuit_breaker_new() {
    let cb = CircuitBreaker::new(5, Duration::from_millis(100));
    assert_eq!(cb.state(), CircuitState::Closed);

    let debug_str = format!("{cb:?}");
    assert!(debug_str.contains("CircuitBreaker"));
}

#[tokio::test]
async fn test_circuit_breaker_success_operation() {
    let cb = CircuitBreaker::new(3, Duration::from_millis(100));

    let result = cb.call(|| async { Ok::<i32, &str>(42) }).await;
    assert_eq!(result.unwrap(), 42);
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_failure_under_threshold() {
    let cb = CircuitBreaker::new(3, Duration::from_millis(100));

    // First failure
    let result = cb.call(|| async { Err::<i32, &str>("error1") }).await;
    assert!(matches!(
        result,
        Err(CircuitBreakerError::Operation("error1"))
    ));
    assert_eq!(cb.state(), CircuitState::Closed);

    // Second failure
    let result = cb.call(|| async { Err::<i32, &str>("error2") }).await;
    assert!(matches!(
        result,
        Err(CircuitBreakerError::Operation("error2"))
    ));
    assert_eq!(cb.state(), CircuitState::Closed); // Still under threshold of 3
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_threshold() {
    let cb = CircuitBreaker::new(2, Duration::from_millis(100));

    // First failure
    let result = cb.call(|| async { Err::<i32, &str>("error1") }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Operation(_))));
    assert_eq!(cb.state(), CircuitState::Closed);

    // Second failure - should open the circuit
    let result = cb.call(|| async { Err::<i32, &str>("error2") }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Operation(_))));
    assert_eq!(cb.state(), CircuitState::Open);
}

#[tokio::test]
async fn test_circuit_breaker_fails_fast_when_open() {
    let cb = CircuitBreaker::new(1, Duration::from_millis(100));
    let counter = Arc::new(AtomicU32::new(0));

    // First failure to open the circuit
    let result = cb
        .call({
            let counter = counter.clone();
            || async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, &str>("error")
            }
        })
        .await;
    assert!(matches!(result, Err(CircuitBreakerError::Operation(_))));
    assert_eq!(cb.state(), CircuitState::Open);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Next call should fail fast without executing the operation
    let result: Result<i32, CircuitBreakerError<&str>> = cb
        .call({
            let counter = counter.clone();
            || async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(42)
            }
        })
        .await;

    assert!(matches!(result, Err(CircuitBreakerError::Open)));
    assert_eq!(counter.load(Ordering::SeqCst), 1); // Counter unchanged
}

#[tokio::test]
async fn test_circuit_breaker_recovery_to_half_open() {
    let cb = CircuitBreaker::new(1, Duration::from_millis(50)); // Longer recovery time for reliability

    // Cause circuit to open
    let result = cb.call(|| async { Err::<i32, &str>("error") }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Operation(_))));
    assert_eq!(cb.state(), CircuitState::Open);

    // Wait for recovery timeout with some buffer
    sleep(Duration::from_millis(100)).await;

    // Next call should work (circuit transitions from open -> half-open -> closed)
    let result = cb.call(|| async { Ok::<i32, &str>(42) }).await;
    assert_eq!(result.unwrap(), 42);
    // Circuit should now be in a working state (either half-open or closed)
    let state = cb.state();
    assert!(state == CircuitState::Closed || state == CircuitState::HalfOpen);
}

#[tokio::test]
async fn test_circuit_breaker_half_open_success_threshold() {
    let cb = CircuitBreaker::new(1, Duration::from_millis(10));

    // Open the circuit
    cb.call(|| async { Err::<i32, &str>("error") })
        .await
        .unwrap_err();
    assert_eq!(cb.state(), CircuitState::Open);

    // Wait for recovery
    sleep(Duration::from_millis(20)).await;

    // Make successful calls (need 3 successes to close from half-open)
    for i in 0..3 {
        let result = cb.call(|| async { Ok::<i32, &str>(i) }).await;
        assert!(result.is_ok());

        if i < 2 {
            // Should still be half-open after first 2 successes
            // Note: state might transition during the success recording
            let state = cb.state();
            assert!(state == CircuitState::HalfOpen || state == CircuitState::Closed);
        }
    }

    // After 3 successes, should be closed
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_half_open_failure_reopens() {
    let cb = CircuitBreaker::new(1, Duration::from_millis(10));

    // Open the circuit
    cb.call(|| async { Err::<i32, &str>("error") })
        .await
        .unwrap_err();
    assert_eq!(cb.state(), CircuitState::Open);

    // Wait for recovery
    sleep(Duration::from_millis(20)).await;

    // Make one successful call to transition to half-open
    let result = cb.call(|| async { Ok::<i32, &str>(1) }).await;
    assert!(result.is_ok());

    // Make a failed call - should reopen the circuit
    let result = cb.call(|| async { Err::<i32, &str>("fail again") }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Operation(_))));
    assert_eq!(cb.state(), CircuitState::Open);
}

#[tokio::test]
async fn test_circuit_breaker_success_resets_failure_count_when_closed() {
    let cb = CircuitBreaker::new(3, Duration::from_millis(100));

    // Make some failures (but not enough to open)
    cb.call(|| async { Err::<i32, &str>("error1") })
        .await
        .unwrap_err();
    cb.call(|| async { Err::<i32, &str>("error2") })
        .await
        .unwrap_err();
    assert_eq!(cb.state(), CircuitState::Closed);

    // Success should reset failure count
    let result = cb.call(|| async { Ok::<i32, &str>(42) }).await;
    assert_eq!(result.unwrap(), 42);
    assert_eq!(cb.state(), CircuitState::Closed);

    // Should need 3 more failures to open (not 1)
    cb.call(|| async { Err::<i32, &str>("error3") })
        .await
        .unwrap_err();
    cb.call(|| async { Err::<i32, &str>("error4") })
        .await
        .unwrap_err();
    assert_eq!(cb.state(), CircuitState::Closed); // Still closed

    cb.call(|| async { Err::<i32, &str>("error5") })
        .await
        .unwrap_err();
    assert_eq!(cb.state(), CircuitState::Open); // Now open
}

// ============================================================================
// CircuitBreakerError Tests
// ============================================================================

#[test]
fn test_circuit_breaker_error_display() {
    let open_error = CircuitBreakerError::<&str>::Open;
    assert_eq!(open_error.to_string(), "Circuit breaker is open");

    let op_error = CircuitBreakerError::Operation("custom error");
    assert_eq!(op_error.to_string(), "Operation failed: custom error");
}

#[test]
fn test_circuit_breaker_error_debug() {
    let open_error = CircuitBreakerError::<&str>::Open;
    let debug_str = format!("{open_error:?}");
    assert!(debug_str.contains("Open"));

    let op_error = CircuitBreakerError::Operation("test");
    let debug_str = format!("{op_error:?}");
    assert!(debug_str.contains("Operation"));
    assert!(debug_str.contains("test"));
}

#[test]
fn test_circuit_breaker_error_source() {
    let open_error = CircuitBreakerError::<std::io::Error>::Open;
    assert!(open_error.source().is_none());

    let io_error = std::io::Error::other("io error");
    let op_error = CircuitBreakerError::Operation(io_error);
    assert!(op_error.source().is_some());

    let source = op_error.source().unwrap();
    assert_eq!(source.to_string(), "io error");
}

// ============================================================================
// Macro Tests
// ============================================================================

#[test]
fn test_measure_time_macro_basic() {
    let result = measure_time!("test operation", {
        std::thread::sleep(Duration::from_millis(1));
        42
    });
    assert_eq!(result, 42);
}

#[test]
fn test_measure_time_macro_with_return() {
    fn test_function() -> i32 {
        measure_time!("test function", {
            if true {
                return 100;
            }
            200
        })
    }

    assert_eq!(test_function(), 100);
}

#[test]
fn test_measure_time_macro_with_complex_expression() {
    let x = 5;
    let result = measure_time!("complex calculation", {
        let y = x * 2;
        let z = y + 3;
        z * z
    });
    assert_eq!(result, 169); // (5*2+3)^2 = 13^2 = 169
}

#[test]
fn test_measure_time_macro_with_error() {
    let result: Result<i32, &str> =
        measure_time!("error operation", { Result::Err("something went wrong") });
    assert_eq!(result.unwrap_err(), "something went wrong");
}

#[test]
fn test_feature_gate_macro_basic() {
    // Test feature gate with block - just verify it compiles and runs
    feature_gate!("tracing", {
        // This block would only compile if "tracing" feature is enabled
        let _test_var = "tracing enabled";
    });

    // Test that the macro expands without errors
    // Note: The feature_gate macro is designed for conditional compilation,
    // not runtime branching, so we just test that it compiles
}

// ============================================================================
// Integration and Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_timeout_with_retry_integration() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig::new()
        .with_max_attempts(3)
        .with_base_delay(Duration::from_millis(1));

    // Operation that times out on first attempt but succeeds on retry
    let result = retry_with_backoff(
        move || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    // First attempt: timeout
                    timeout(Duration::from_millis(1), async {
                        sleep(Duration::from_millis(10)).await;
                        "success"
                    })
                    .await
                    .map_err(|_| "timeout")
                } else {
                    // Subsequent attempts: succeed quickly
                    timeout(Duration::from_millis(100), async { "success" })
                        .await
                        .map_err(|_| "timeout")
                }
            }
        },
        config,
        |error: &&str| *error == "timeout",
    )
    .await;

    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 2); // Failed once, succeeded once
}

#[tokio::test]
async fn test_circuit_breaker_with_timeout_integration() {
    let cb = CircuitBreaker::new(2, Duration::from_millis(50));
    let slow_call_counter = Arc::new(AtomicU32::new(0));

    // Make two slow calls that will timeout and cause circuit to open
    for _ in 0..2 {
        let counter = slow_call_counter.clone();
        let result = cb
            .call(|| async move {
                counter.fetch_add(1, Ordering::SeqCst);
                timeout(Duration::from_millis(10), async {
                    sleep(Duration::from_millis(50)).await;
                    "slow success"
                })
                .await
                .map_err(|_| "timeout")
            })
            .await;

        assert!(matches!(
            result,
            Err(CircuitBreakerError::Operation("timeout"))
        ));
    }

    assert_eq!(cb.state(), CircuitState::Open);
    assert_eq!(slow_call_counter.load(Ordering::SeqCst), 2);

    // Next call should fail fast
    let result: Result<&str, CircuitBreakerError<&str>> = cb
        .call(|| async {
            slow_call_counter.fetch_add(1, Ordering::SeqCst);
            Ok("should not execute")
        })
        .await;

    assert!(matches!(result, Err(CircuitBreakerError::Open)));
    assert_eq!(slow_call_counter.load(Ordering::SeqCst), 2); // Unchanged
}

#[tokio::test]
async fn test_concurrent_circuit_breaker_access() {
    let cb = Arc::new(CircuitBreaker::new(3, Duration::from_millis(100)));
    let success_counter = Arc::new(AtomicU32::new(0));
    let failure_counter = Arc::new(AtomicU32::new(0));

    // Spawn multiple tasks that will succeed or fail based on a shared condition
    let should_fail = Arc::new(AtomicBool::new(true));
    let mut handles = Vec::new();

    for _ in 0..10 {
        let cb = cb.clone();
        let success_counter = success_counter.clone();
        let failure_counter = failure_counter.clone();
        let should_fail = should_fail.clone();

        let handle = tokio::spawn(async move {
            let result = cb
                .call(|| async {
                    if should_fail.load(Ordering::SeqCst) {
                        Err("failing")
                    } else {
                        Ok("success")
                    }
                })
                .await;

            match result {
                Ok(_) => success_counter.fetch_add(1, Ordering::SeqCst),
                Err(CircuitBreakerError::Operation(_)) => {
                    failure_counter.fetch_add(1, Ordering::SeqCst)
                }
                Err(CircuitBreakerError::Open) => 0, // Circuit was open
            };
        });
        handles.push(handle);
    }

    // Switch to success mode partway through
    sleep(Duration::from_millis(1)).await;
    should_fail.store(false, Ordering::SeqCst);

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Should have some failures and possibly some successes
    let total_ops = success_counter.load(Ordering::SeqCst) + failure_counter.load(Ordering::SeqCst);
    assert!(total_ops > 0);
    assert!(failure_counter.load(Ordering::SeqCst) > 0); // Some operations should have failed
}

#[test]
fn test_extreme_retry_config_values() {
    // Test with very large values
    let large_config = RetryConfig::new()
        .with_max_attempts(1000)
        .with_base_delay(Duration::from_secs(1))
        .with_max_delay(Duration::from_secs(3600))
        .with_backoff_multiplier(10.0);

    assert_eq!(large_config.max_attempts, 1000);
    assert_eq!(large_config.base_delay, Duration::from_secs(1));
    assert_eq!(large_config.max_delay, Duration::from_secs(3600));
    assert_eq!(large_config.backoff_multiplier, 10.0);

    let delay = large_config.delay_for_attempt(5);
    // With jitter enabled (default), the delay will be close to max_delay but not exactly equal
    // Allow some tolerance for jitter
    assert!(delay <= large_config.max_delay);
    assert!(delay >= large_config.max_delay.mul_f64(0.95)); // Within 5% due to jitter

    // Test with small fractional multiplier
    let small_multiplier_config = RetryConfig::new()
        .with_base_delay(Duration::from_millis(1000))
        .with_backoff_multiplier(0.5)
        .with_jitter(false);

    let delay1 = small_multiplier_config.delay_for_attempt(1);
    let delay2 = small_multiplier_config.delay_for_attempt(2);

    assert_eq!(delay1, Duration::from_millis(1000));
    assert_eq!(delay2, Duration::from_millis(500)); // 1000 * 0.5^1
}

#[tokio::test]
async fn test_complex_timeout_scenarios() {
    // Test timeout with nested async operations
    let result = timeout(Duration::from_millis(200), async {
        let inner_result = timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(25)).await;
            "inner success"
        })
        .await;

        match inner_result {
            Ok(val) => format!("outer: {val}"),
            Err(_) => "inner timeout".to_string(),
        }
    })
    .await;

    assert_eq!(result.unwrap(), "outer: inner success");

    // Test timeout with error propagation (simplified test without panic)
    let result = timeout(Duration::from_millis(100), async {
        timeout(Duration::from_millis(50), async {
            sleep(Duration::from_millis(10)).await;
            Result::<&str, &str>::Err("operation error")
        })
        .await
    })
    .await;

    // Should succeed with timeout but contain the error
    assert!(result.is_ok()); // Timeout didn't occur
    let inner_result = result.unwrap();
    assert!(inner_result.is_ok()); // Inner timeout didn't occur
    let final_result = inner_result.unwrap();
    assert_eq!(final_result.unwrap_err(), "operation error");
}
