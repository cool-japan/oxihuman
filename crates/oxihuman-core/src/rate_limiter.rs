// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Token bucket rate limiter.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub capacity: f32,
    pub refill_rate: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RateLimiter {
    pub tokens: f32,
    pub config: RateLimiterConfig,
}

#[allow(dead_code)]
pub fn default_rate_limiter_config() -> RateLimiterConfig {
    RateLimiterConfig { capacity: 10.0, refill_rate: 1.0 }
}

#[allow(dead_code)]
pub fn new_rate_limiter(config: RateLimiterConfig) -> RateLimiter {
    let tokens = config.capacity;
    RateLimiter { tokens, config }
}

#[allow(dead_code)]
pub fn rl_try_acquire(rl: &mut RateLimiter, cost: f32) -> bool {
    if rl.tokens >= cost {
        rl.tokens -= cost;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn rl_refill(rl: &mut RateLimiter, dt: f32) {
    rl.tokens = (rl.tokens + rl.config.refill_rate * dt).min(rl.config.capacity);
}

#[allow(dead_code)]
pub fn rl_tokens(rl: &RateLimiter) -> f32 {
    rl.tokens
}

#[allow(dead_code)]
pub fn rl_is_full(rl: &RateLimiter) -> bool {
    rl.tokens >= rl.config.capacity
}

#[allow(dead_code)]
pub fn rl_fill_ratio(rl: &RateLimiter) -> f32 {
    if rl.config.capacity <= 0.0 {
        return 0.0;
    }
    rl.tokens / rl.config.capacity
}

#[allow(dead_code)]
pub fn rl_reset(rl: &mut RateLimiter) {
    rl.tokens = rl.config.capacity;
}

#[allow(dead_code)]
pub fn rl_set_capacity(rl: &mut RateLimiter, capacity: f32) {
    rl.config.capacity = capacity;
    rl.tokens = rl.tokens.min(capacity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_full() {
        let rl = new_rate_limiter(default_rate_limiter_config());
        assert!(rl_is_full(&rl));
    }

    #[test]
    fn test_acquire_succeeds_with_tokens() {
        let mut rl = new_rate_limiter(default_rate_limiter_config());
        assert!(rl_try_acquire(&mut rl, 5.0));
    }

    #[test]
    fn test_acquire_fails_insufficient_tokens() {
        let mut rl = new_rate_limiter(default_rate_limiter_config());
        rl_try_acquire(&mut rl, 10.0);
        assert!(!rl_try_acquire(&mut rl, 1.0));
    }

    #[test]
    fn test_refill_increases_tokens() {
        let mut rl = new_rate_limiter(default_rate_limiter_config());
        rl_try_acquire(&mut rl, 5.0);
        let before = rl_tokens(&rl);
        rl_refill(&mut rl, 2.0);
        assert!(rl_tokens(&rl) > before);
    }

    #[test]
    fn test_refill_caps_at_capacity() {
        let mut rl = new_rate_limiter(default_rate_limiter_config());
        rl_refill(&mut rl, 1000.0);
        assert!((rl_tokens(&rl) - rl.config.capacity).abs() < 1e-6);
    }

    #[test]
    fn test_fill_ratio() {
        let mut rl = new_rate_limiter(default_rate_limiter_config());
        rl_try_acquire(&mut rl, 5.0);
        let ratio = rl_fill_ratio(&rl);
        assert!((ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut rl = new_rate_limiter(default_rate_limiter_config());
        rl_try_acquire(&mut rl, 8.0);
        rl_reset(&mut rl);
        assert!(rl_is_full(&rl));
    }

    #[test]
    fn test_set_capacity() {
        let mut rl = new_rate_limiter(default_rate_limiter_config());
        rl_set_capacity(&mut rl, 5.0);
        assert!((rl.config.capacity - 5.0).abs() < 1e-6);
        assert!(rl_tokens(&rl) <= 5.0);
    }
}
