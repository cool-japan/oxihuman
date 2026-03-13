// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sliding window rate limiter — tracks request timestamps in a ring buffer.

/// Configuration for the sliding window rate limiter.
#[derive(Clone, Debug)]
pub struct SlidingRateLimiterConfig {
    /// Maximum number of requests allowed in the window.
    pub max_requests: usize,
    /// Window size in milliseconds.
    pub window_ms: u64,
}

impl Default for SlidingRateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_ms: 1000,
        }
    }
}

/// A sliding window rate limiter.
pub struct SlidingRateLimiter {
    pub config: SlidingRateLimiterConfig,
    timestamps: Vec<u64>,
}

/// Creates a new sliding window rate limiter.
pub fn new_rate_limiter(config: SlidingRateLimiterConfig) -> SlidingRateLimiter {
    SlidingRateLimiter {
        config,
        timestamps: Vec::new(),
    }
}

/// Returns true if the request is allowed, recording the timestamp.
pub fn check_and_record(limiter: &mut SlidingRateLimiter, now_ms: u64) -> bool {
    evict_old(limiter, now_ms);
    if limiter.timestamps.len() < limiter.config.max_requests {
        limiter.timestamps.push(now_ms);
        true
    } else {
        false
    }
}

/// Removes timestamps outside the current window.
pub fn evict_old(limiter: &mut SlidingRateLimiter, now_ms: u64) {
    let cutoff = now_ms.saturating_sub(limiter.config.window_ms);
    limiter.timestamps.retain(|&t| t >= cutoff);
}

/// Returns the number of requests recorded in the current window.
pub fn requests_in_window(limiter: &SlidingRateLimiter, now_ms: u64) -> usize {
    let cutoff = now_ms.saturating_sub(limiter.config.window_ms);
    limiter.timestamps.iter().filter(|&&t| t >= cutoff).count()
}

/// Returns the remaining request budget for the current window.
pub fn remaining_budget(limiter: &SlidingRateLimiter, now_ms: u64) -> usize {
    let used = requests_in_window(limiter, now_ms);
    limiter.config.max_requests.saturating_sub(used)
}

/// Resets the limiter, clearing all recorded timestamps.
pub fn reset_limiter(limiter: &mut SlidingRateLimiter) {
    limiter.timestamps.clear();
}

impl SlidingRateLimiter {
    /// Creates a new rate limiter with default config.
    pub fn new(config: SlidingRateLimiterConfig) -> Self {
        new_rate_limiter(config)
    }

    /// Returns true if request at `now_ms` is within the rate limit.
    pub fn allow(&mut self, now_ms: u64) -> bool {
        check_and_record(self, now_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limiter(max: usize, window_ms: u64) -> SlidingRateLimiter {
        new_rate_limiter(SlidingRateLimiterConfig {
            max_requests: max,
            window_ms,
        })
    }

    #[test]
    fn test_allows_up_to_limit() {
        let mut lim = make_limiter(3, 1000);
        assert!(check_and_record(&mut lim, 0));
        assert!(check_and_record(&mut lim, 100));
        assert!(check_and_record(&mut lim, 200));
        /* fourth request should be denied */
        assert!(!check_and_record(&mut lim, 300));
    }

    #[test]
    fn test_window_slides_allowing_new_requests() {
        let mut lim = make_limiter(2, 1000);
        check_and_record(&mut lim, 0);
        check_and_record(&mut lim, 100);
        /* slide window past first two requests */
        assert!(check_and_record(&mut lim, 1100));
    }

    #[test]
    fn test_evict_old_removes_entries() {
        let mut lim = make_limiter(10, 1000);
        check_and_record(&mut lim, 0);
        evict_old(&mut lim, 2000);
        assert_eq!(requests_in_window(&lim, 2000), 0);
    }

    #[test]
    fn test_remaining_budget_decreases() {
        let mut lim = make_limiter(5, 1000);
        assert_eq!(remaining_budget(&lim, 0), 5);
        check_and_record(&mut lim, 0);
        assert_eq!(remaining_budget(&lim, 0), 4);
    }

    #[test]
    fn test_reset_clears_timestamps() {
        let mut lim = make_limiter(3, 1000);
        check_and_record(&mut lim, 0);
        check_and_record(&mut lim, 50);
        reset_limiter(&mut lim);
        assert_eq!(requests_in_window(&lim, 100), 0);
    }

    #[test]
    fn test_requests_in_window_counts_correctly() {
        let mut lim = make_limiter(10, 500);
        check_and_record(&mut lim, 0);
        check_and_record(&mut lim, 100);
        /* t=600: inside window at now=700 (cutoff=200, 600>=200).
         * t=0 is evicted when t=600 is recorded (evict_old cutoff=100, 0<100).
         * t=100 is outside window at now=700 (100<200). Only t=600 counts. */
        check_and_record(&mut lim, 600);
        assert_eq!(requests_in_window(&lim, 700), 1);
    }

    #[test]
    fn test_allow_method_delegates() {
        let mut lim = SlidingRateLimiter::new(SlidingRateLimiterConfig {
            max_requests: 2,
            window_ms: 500,
        });
        assert!(lim.allow(0));
        assert!(lim.allow(100));
        assert!(!lim.allow(200));
    }

    #[test]
    fn test_saturating_sub_no_underflow() {
        /* window_ms larger than now_ms => cutoff should be 0 not wraparound */
        let mut lim = make_limiter(5, 5000);
        check_and_record(&mut lim, 10);
        assert_eq!(requests_in_window(&lim, 50), 1);
    }

    #[test]
    fn test_zero_limit_denies_all() {
        let mut lim = make_limiter(0, 1000);
        assert!(!check_and_record(&mut lim, 100));
    }
}
