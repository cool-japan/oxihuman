//! Token bucket rate limiter — controls the rate of asset loads, network requests,
//! and other throttled operations using the token bucket algorithm.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TokenBucketConfig {
    pub capacity: f32,
    pub refill_rate: f32,
    pub initial_tokens: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TokenBucket {
    pub config: TokenBucketConfig,
    tokens: f32,
}

#[allow(dead_code)]
pub fn default_token_bucket_config() -> TokenBucketConfig {
    TokenBucketConfig {
        capacity: 100.0,
        refill_rate: 10.0, // tokens per second
        initial_tokens: 100.0,
    }
}

#[allow(dead_code)]
pub fn new_token_bucket(config: TokenBucketConfig) -> TokenBucket {
    let tokens = config.initial_tokens.clamp(0.0, config.capacity);
    TokenBucket { tokens, config }
}

/// Consume `amount` tokens. Blocks (returns false) if insufficient tokens.
#[allow(dead_code)]
pub fn bucket_consume(bucket: &mut TokenBucket, amount: f32) -> bool {
    if bucket.tokens >= amount {
        bucket.tokens -= amount;
        true
    } else {
        false
    }
}

/// Try to consume up to `amount` tokens; returns how many were actually consumed.
#[allow(dead_code)]
pub fn bucket_try_consume(bucket: &mut TokenBucket, amount: f32) -> f32 {
    let available = bucket.tokens.min(amount);
    bucket.tokens -= available;
    available
}

/// Refill the bucket as if `delta_seconds` of time have passed.
#[allow(dead_code)]
pub fn bucket_refill(bucket: &mut TokenBucket, delta_seconds: f32) {
    bucket.tokens =
        (bucket.tokens + bucket.config.refill_rate * delta_seconds).min(bucket.config.capacity);
}

#[allow(dead_code)]
pub fn bucket_available(bucket: &TokenBucket) -> f32 {
    bucket.tokens
}

#[allow(dead_code)]
pub fn bucket_capacity(bucket: &TokenBucket) -> f32 {
    bucket.config.capacity
}

#[allow(dead_code)]
pub fn bucket_is_full(bucket: &TokenBucket) -> bool {
    (bucket.tokens - bucket.config.capacity).abs() < 1e-6
}

#[allow(dead_code)]
pub fn bucket_to_json(bucket: &TokenBucket) -> String {
    format!(
        "{{\"capacity\":{:.4},\"tokens\":{:.4},\"refill_rate\":{:.4}}}",
        bucket.config.capacity, bucket.tokens, bucket.config.refill_rate
    )
}

#[allow(dead_code)]
pub fn bucket_reset(bucket: &mut TokenBucket) {
    bucket.tokens = bucket.config.initial_tokens.clamp(0.0, bucket.config.capacity);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bucket() -> TokenBucket {
        new_token_bucket(default_token_bucket_config())
    }

    #[test]
    fn test_initial_tokens_full() {
        let b = make_bucket();
        assert!((bucket_available(&b) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_consume_success() {
        let mut b = make_bucket();
        assert!(bucket_consume(&mut b, 50.0));
        assert!((bucket_available(&b) - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_consume_fail_insufficient() {
        let mut b = make_bucket();
        assert!(!bucket_consume(&mut b, 200.0));
        assert!((bucket_available(&b) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_try_consume_partial() {
        let mut b = make_bucket();
        bucket_consume(&mut b, 80.0);
        let consumed = bucket_try_consume(&mut b, 50.0);
        assert!((consumed - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_refill() {
        let mut b = make_bucket();
        bucket_consume(&mut b, 100.0);
        bucket_refill(&mut b, 1.0); // 10 tokens/sec * 1s = 10 tokens
        assert!((bucket_available(&b) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_refill_does_not_exceed_capacity() {
        let mut b = make_bucket();
        bucket_refill(&mut b, 1000.0);
        assert!(bucket_available(&b) <= bucket_capacity(&b) + 1e-5);
    }

    #[test]
    fn test_is_full() {
        let b = make_bucket();
        assert!(bucket_is_full(&b));
    }

    #[test]
    fn test_reset() {
        let mut b = make_bucket();
        bucket_consume(&mut b, 80.0);
        bucket_reset(&mut b);
        assert!((bucket_available(&b) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let b = make_bucket();
        let j = bucket_to_json(&b);
        assert!(j.contains("capacity"));
        assert!(j.contains("tokens"));
        assert!(j.contains("refill_rate"));
    }

    #[test]
    fn test_capacity() {
        let b = make_bucket();
        assert!((bucket_capacity(&b) - 100.0).abs() < 1e-5);
    }
}
