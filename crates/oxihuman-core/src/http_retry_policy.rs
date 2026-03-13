// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HTTP retry policy with exponential backoff and jitter.

/// Strategy for retry backoff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackoffStrategy {
    Fixed,
    Exponential,
    LinearJitter,
}

/// Configuration for a retry policy.
#[derive(Clone, Debug)]
pub struct HttpRetryPolicyConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub strategy: BackoffStrategy,
    pub jitter_ms: u64,
}

impl Default for HttpRetryPolicyConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 30_000,
            strategy: BackoffStrategy::Exponential,
            jitter_ms: 50,
        }
    }
}

/// Tracks retry state for a single operation.
pub struct RetryState {
    pub config: HttpRetryPolicyConfig,
    pub attempt: u32,
}

/// Creates a new retry state from config.
pub fn new_retry_state(config: HttpRetryPolicyConfig) -> RetryState {
    RetryState { config, attempt: 0 }
}

/// Computes the delay in ms before the next retry attempt.
pub fn delay_for_attempt(config: &HttpRetryPolicyConfig, attempt: u32) -> u64 {
    let raw = match config.strategy {
        BackoffStrategy::Fixed => config.base_delay_ms,
        BackoffStrategy::Exponential => {
            let shift = attempt.min(30);
            let factor = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
            config.base_delay_ms.saturating_mul(factor)
        }
        BackoffStrategy::LinearJitter => config.base_delay_ms.saturating_mul(attempt as u64 + 1),
    };
    raw.min(config.max_delay_ms)
        .saturating_add(config.jitter_ms)
}

/// Returns true if another retry is possible.
pub fn can_retry(state: &RetryState) -> bool {
    state.attempt < state.config.max_attempts
}

/// Records that an attempt was made, returning the delay before the next retry.
pub fn next_delay_ms(state: &mut RetryState) -> Option<u64> {
    if !can_retry(state) {
        return None;
    }
    let delay = delay_for_attempt(&state.config, state.attempt);
    state.attempt += 1;
    Some(delay)
}

/// Resets the retry state for a new operation.
pub fn reset_retry(state: &mut RetryState) {
    state.attempt = 0;
}

/// Returns the total remaining attempts.
pub fn remaining_attempts(state: &RetryState) -> u32 {
    state.config.max_attempts.saturating_sub(state.attempt)
}

impl RetryState {
    /// Creates a new retry state with default config.
    pub fn new(config: HttpRetryPolicyConfig) -> Self {
        new_retry_state(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(max: u32, strategy: BackoffStrategy) -> RetryState {
        new_retry_state(HttpRetryPolicyConfig {
            max_attempts: max,
            base_delay_ms: 100,
            max_delay_ms: 10_000,
            strategy,
            jitter_ms: 0,
        })
    }

    #[test]
    fn test_can_retry_when_attempts_remain() {
        let state = make_state(3, BackoffStrategy::Fixed);
        assert!(can_retry(&state));
    }

    #[test]
    fn test_cannot_retry_when_exhausted() {
        let mut state = make_state(2, BackoffStrategy::Fixed);
        next_delay_ms(&mut state);
        next_delay_ms(&mut state);
        assert!(!can_retry(&state));
    }

    #[test]
    fn test_fixed_delay_constant() {
        let cfg = HttpRetryPolicyConfig {
            strategy: BackoffStrategy::Fixed,
            jitter_ms: 0,
            ..Default::default()
        };
        let d0 = delay_for_attempt(&cfg, 0);
        let d1 = delay_for_attempt(&cfg, 1);
        assert_eq!(d0, d1);
    }

    #[test]
    fn test_exponential_delay_grows() {
        let cfg = HttpRetryPolicyConfig {
            strategy: BackoffStrategy::Exponential,
            jitter_ms: 0,
            ..Default::default()
        };
        let d0 = delay_for_attempt(&cfg, 0);
        let d1 = delay_for_attempt(&cfg, 1);
        assert!(d1 > d0);
    }

    #[test]
    fn test_delay_capped_at_max() {
        let cfg = HttpRetryPolicyConfig {
            max_delay_ms: 200,
            base_delay_ms: 100,
            strategy: BackoffStrategy::Exponential,
            jitter_ms: 0,
            max_attempts: 10,
        };
        let d = delay_for_attempt(&cfg, 20);
        assert!(d <= 200);
    }

    #[test]
    fn test_next_delay_ms_returns_none_when_exhausted() {
        let mut state = make_state(1, BackoffStrategy::Fixed);
        let _ = next_delay_ms(&mut state);
        assert!(next_delay_ms(&mut state).is_none());
    }

    #[test]
    fn test_reset_restores_full_budget() {
        let mut state = make_state(3, BackoffStrategy::Fixed);
        next_delay_ms(&mut state);
        next_delay_ms(&mut state);
        reset_retry(&mut state);
        assert_eq!(remaining_attempts(&state), 3);
    }

    #[test]
    fn test_remaining_attempts_decrements() {
        let mut state = make_state(3, BackoffStrategy::Fixed);
        assert_eq!(remaining_attempts(&state), 3);
        next_delay_ms(&mut state);
        assert_eq!(remaining_attempts(&state), 2);
    }

    #[test]
    fn test_linear_jitter_grows_linearly() {
        let cfg = HttpRetryPolicyConfig {
            strategy: BackoffStrategy::LinearJitter,
            jitter_ms: 0,
            ..Default::default()
        };
        let d0 = delay_for_attempt(&cfg, 0);
        let d2 = delay_for_attempt(&cfg, 2);
        assert!(d2 > d0);
    }
}
