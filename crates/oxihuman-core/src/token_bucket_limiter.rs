// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct TokenBucketLimiter {
    pub tokens: f32,
    pub capacity: f32,
    pub refill_rate: f32,
}

impl TokenBucketLimiter {
    pub fn new(capacity: f32, refill_rate: f32) -> Self {
        TokenBucketLimiter {
            tokens: capacity,
            capacity,
            refill_rate,
        }
    }
}

pub fn new_rate_limiter_tb(capacity: f32, refill_rate: f32) -> TokenBucketLimiter {
    TokenBucketLimiter::new(capacity, refill_rate)
}

pub fn limiter_refill(r: &mut TokenBucketLimiter, dt: f32) {
    r.tokens = (r.tokens + r.refill_rate * dt).min(r.capacity);
}

pub fn limiter_consume(r: &mut TokenBucketLimiter, tokens: f32) -> bool {
    if r.tokens >= tokens {
        r.tokens -= tokens;
        true
    } else {
        false
    }
}

pub fn limiter_available(r: &TokenBucketLimiter) -> f32 {
    r.tokens
}

pub fn limiter_is_full(r: &TokenBucketLimiter) -> bool {
    r.tokens >= r.capacity
}

pub fn limiter_reset(r: &mut TokenBucketLimiter) {
    r.tokens = r.capacity;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_full() {
        /* new limiter starts full */
        let r = new_rate_limiter_tb(10.0, 1.0);
        assert!(limiter_is_full(&r));
    }

    #[test]
    fn test_consume_success() {
        /* consume succeeds when tokens available */
        let mut r = new_rate_limiter_tb(10.0, 1.0);
        assert!(limiter_consume(&mut r, 5.0));
        assert!((limiter_available(&r) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_consume_fail() {
        /* consume fails when insufficient tokens */
        let mut r = new_rate_limiter_tb(5.0, 1.0);
        limiter_consume(&mut r, 5.0);
        assert!(!limiter_consume(&mut r, 1.0));
    }

    #[test]
    fn test_refill() {
        /* refill increases token count */
        let mut r = new_rate_limiter_tb(10.0, 2.0);
        limiter_consume(&mut r, 4.0);
        limiter_refill(&mut r, 1.0);
        assert!((limiter_available(&r) - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_refill_capped() {
        /* refill does not exceed capacity */
        let mut r = new_rate_limiter_tb(10.0, 5.0);
        limiter_refill(&mut r, 100.0);
        assert!((limiter_available(&r) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        /* reset restores full capacity */
        let mut r = new_rate_limiter_tb(10.0, 1.0);
        limiter_consume(&mut r, 8.0);
        limiter_reset(&mut r);
        assert!(limiter_is_full(&r));
    }

    #[test]
    fn test_available() {
        /* available reflects current tokens */
        let r = new_rate_limiter_tb(7.0, 1.0);
        assert!((limiter_available(&r) - 7.0).abs() < 1e-6);
    }
}
