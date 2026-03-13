// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Circuit breaker state machine — open/closed/half-open transitions.

/// State of the circuit breaker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Configuration for the circuit breaker.
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit.
    pub failure_threshold: u32,
    /// Number of consecutive successes in half-open to close.
    pub success_threshold: u32,
    /// How long to wait (ms) before moving from Open to HalfOpen.
    pub reset_timeout_ms: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout_ms: 5000,
        }
    }
}

/// The circuit breaker instance.
pub struct CircuitBreaker {
    pub config: CircuitBreakerConfig,
    pub state: CircuitState,
    consecutive_failures: u32,
    consecutive_successes: u32,
    last_opened_at: u64,
}

/// Creates a new circuit breaker in the Closed state.
pub fn new_circuit_breaker(config: CircuitBreakerConfig) -> CircuitBreaker {
    CircuitBreaker {
        config,
        state: CircuitState::Closed,
        consecutive_failures: 0,
        consecutive_successes: 0,
        last_opened_at: 0,
    }
}

/// Returns true if the circuit allows a request at the given timestamp.
pub fn is_request_allowed(cb: &mut CircuitBreaker, now_ms: u64) -> bool {
    match cb.state {
        CircuitState::Closed => true,
        CircuitState::HalfOpen => true,
        CircuitState::Open => {
            if now_ms.saturating_sub(cb.last_opened_at) >= cb.config.reset_timeout_ms {
                cb.state = CircuitState::HalfOpen;
                cb.consecutive_successes = 0;
                true
            } else {
                false
            }
        }
    }
}

/// Records a successful call, potentially closing the circuit.
pub fn record_success(cb: &mut CircuitBreaker) {
    cb.consecutive_failures = 0;
    if cb.state == CircuitState::HalfOpen {
        cb.consecutive_successes += 1;
        if cb.consecutive_successes >= cb.config.success_threshold {
            cb.state = CircuitState::Closed;
        }
    }
}

/// Records a failed call, potentially opening the circuit.
pub fn record_failure(cb: &mut CircuitBreaker, now_ms: u64) {
    cb.consecutive_successes = 0;
    cb.consecutive_failures += 1;
    if cb.consecutive_failures >= cb.config.failure_threshold || cb.state == CircuitState::HalfOpen
    {
        cb.state = CircuitState::Open;
        cb.last_opened_at = now_ms;
        cb.consecutive_failures = 0;
    }
}

/// Returns the current state of the circuit breaker.
pub fn current_state(cb: &CircuitBreaker) -> CircuitState {
    cb.state
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with default config.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        new_circuit_breaker(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cb() -> CircuitBreaker {
        new_circuit_breaker(CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            reset_timeout_ms: 1000,
        })
    }

    #[test]
    fn test_initial_state_is_closed() {
        let cb = make_cb();
        assert_eq!(current_state(&cb), CircuitState::Closed);
    }

    #[test]
    fn test_opens_after_threshold_failures() {
        let mut cb = make_cb();
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        assert_eq!(current_state(&cb), CircuitState::Open);
    }

    #[test]
    fn test_open_circuit_denies_requests() {
        let mut cb = make_cb();
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        assert!(!is_request_allowed(&mut cb, 500));
    }

    #[test]
    fn test_moves_to_half_open_after_timeout() {
        let mut cb = make_cb();
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        is_request_allowed(&mut cb, 1500); /* past reset_timeout */
        assert_eq!(current_state(&cb), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_closes_after_successes() {
        let mut cb = make_cb();
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        is_request_allowed(&mut cb, 1500);
        record_success(&mut cb);
        record_success(&mut cb);
        assert_eq!(current_state(&cb), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_failure_reopens() {
        let mut cb = make_cb();
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        is_request_allowed(&mut cb, 1500);
        record_failure(&mut cb, 1500); /* fail in half-open => reopen */
        assert_eq!(current_state(&cb), CircuitState::Open);
    }

    #[test]
    fn test_success_in_closed_resets_failure_count() {
        let mut cb = make_cb();
        record_failure(&mut cb, 0);
        record_failure(&mut cb, 0);
        record_success(&mut cb);
        record_failure(&mut cb, 0); /* only 1 failure now => still closed */
        assert_eq!(current_state(&cb), CircuitState::Closed);
    }

    #[test]
    fn test_closed_allows_requests() {
        let mut cb = make_cb();
        assert!(is_request_allowed(&mut cb, 0));
    }

    #[test]
    fn test_open_not_expired_denies() {
        let mut cb = make_cb();
        record_failure(&mut cb, 1000);
        record_failure(&mut cb, 1000);
        record_failure(&mut cb, 1000);
        /* now = 1500, timeout = 1000 => not yet expired from opened_at=1000 */
        assert!(!is_request_allowed(&mut cb, 1999));
    }
}
