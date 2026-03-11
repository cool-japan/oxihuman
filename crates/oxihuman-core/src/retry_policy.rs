#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Retry policy for transient failure handling.

/// Result of a retry check.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum RetryResult {
    Retry,
    GiveUp,
}

/// Retry policy configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RetryPolicy {
    max_retries: u32,
    base_delay_ms: u64,
    attempt: u32,
}

/// Create a new `RetryPolicy`.
#[allow(dead_code)]
pub fn new_retry_policy(max_retries: u32, base_delay_ms: u64) -> RetryPolicy {
    RetryPolicy {
        max_retries,
        base_delay_ms,
        attempt: 0,
    }
}

/// Check whether another retry should be attempted, and advance the attempt counter.
#[allow(dead_code)]
pub fn should_retry(policy: &mut RetryPolicy) -> RetryResult {
    if policy.attempt < policy.max_retries {
        policy.attempt += 1;
        RetryResult::Retry
    } else {
        RetryResult::GiveUp
    }
}

/// Return the current attempt number (1-based after first retry).
#[allow(dead_code)]
pub fn retry_count(policy: &RetryPolicy) -> u32 {
    policy.attempt
}

/// Return the maximum number of retries configured.
#[allow(dead_code)]
pub fn max_retries(policy: &RetryPolicy) -> u32 {
    policy.max_retries
}

/// Return the base delay in milliseconds.
#[allow(dead_code)]
pub fn retry_delay_ms(policy: &RetryPolicy) -> u64 {
    policy.base_delay_ms
}

/// Return the delay for the current attempt using exponential backoff.
#[allow(dead_code)]
pub fn retry_with_backoff(policy: &RetryPolicy) -> u64 {
    if policy.attempt == 0 {
        return 0;
    }
    policy
        .base_delay_ms
        .saturating_mul(1u64 << (policy.attempt - 1).min(10))
}

/// Reset the retry counter to zero.
#[allow(dead_code)]
pub fn reset_retry(policy: &mut RetryPolicy) {
    policy.attempt = 0;
}

/// Return true if all retries are exhausted.
#[allow(dead_code)]
pub fn retry_exhausted(policy: &RetryPolicy) -> bool {
    policy.attempt >= policy.max_retries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_retry_policy() {
        let p = new_retry_policy(3, 100);
        assert_eq!(max_retries(&p), 3);
        assert_eq!(retry_delay_ms(&p), 100);
        assert_eq!(retry_count(&p), 0);
    }

    #[test]
    fn test_should_retry() {
        let mut p = new_retry_policy(2, 50);
        assert_eq!(should_retry(&mut p), RetryResult::Retry);
        assert_eq!(should_retry(&mut p), RetryResult::Retry);
        assert_eq!(should_retry(&mut p), RetryResult::GiveUp);
    }

    #[test]
    fn test_retry_exhausted() {
        let mut p = new_retry_policy(1, 0);
        assert!(!retry_exhausted(&p));
        should_retry(&mut p);
        assert!(retry_exhausted(&p));
    }

    #[test]
    fn test_retry_count() {
        let mut p = new_retry_policy(5, 10);
        should_retry(&mut p);
        should_retry(&mut p);
        assert_eq!(retry_count(&p), 2);
    }

    #[test]
    fn test_reset_retry() {
        let mut p = new_retry_policy(3, 10);
        should_retry(&mut p);
        should_retry(&mut p);
        reset_retry(&mut p);
        assert_eq!(retry_count(&p), 0);
    }

    #[test]
    fn test_retry_with_backoff() {
        let mut p = new_retry_policy(5, 100);
        assert_eq!(retry_with_backoff(&p), 0);
        should_retry(&mut p);
        assert_eq!(retry_with_backoff(&p), 100);
        should_retry(&mut p);
        assert_eq!(retry_with_backoff(&p), 200);
        should_retry(&mut p);
        assert_eq!(retry_with_backoff(&p), 400);
    }

    #[test]
    fn test_zero_retries() {
        let mut p = new_retry_policy(0, 0);
        assert_eq!(should_retry(&mut p), RetryResult::GiveUp);
    }

    #[test]
    fn test_max_retries_accessor() {
        let p = new_retry_policy(7, 0);
        assert_eq!(max_retries(&p), 7);
    }
}
