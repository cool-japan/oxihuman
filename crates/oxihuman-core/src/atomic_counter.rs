#![allow(dead_code)]

use std::sync::atomic::{AtomicI64, Ordering};

/// A thread-safe atomic counter.
#[allow(dead_code)]
pub struct AtomicCounter {
    value: AtomicI64,
}

/// Creates a new atomic counter with value 0.
#[allow(dead_code)]
pub fn new_atomic_counter() -> AtomicCounter {
    AtomicCounter {
        value: AtomicI64::new(0),
    }
}

/// Increments by 1 and returns the new value.
#[allow(dead_code)]
pub fn counter_increment(counter: &AtomicCounter) -> i64 {
    counter.value.fetch_add(1, Ordering::SeqCst) + 1
}

/// Decrements by 1 and returns the new value.
#[allow(dead_code)]
pub fn counter_decrement(counter: &AtomicCounter) -> i64 {
    counter.value.fetch_sub(1, Ordering::SeqCst) - 1
}

/// Returns the current value.
#[allow(dead_code)]
pub fn counter_value(counter: &AtomicCounter) -> i64 {
    counter.value.load(Ordering::SeqCst)
}

/// Resets to 0.
#[allow(dead_code)]
pub fn counter_reset(counter: &AtomicCounter) {
    counter.value.store(0, Ordering::SeqCst);
}

/// Adds the given amount and returns the new value.
#[allow(dead_code)]
pub fn counter_add(counter: &AtomicCounter, amount: i64) -> i64 {
    counter.value.fetch_add(amount, Ordering::SeqCst) + amount
}

/// Compare-and-exchange: if current == expected, set to new_val. Returns old value.
#[allow(dead_code)]
pub fn counter_compare_exchange(counter: &AtomicCounter, expected: i64, new_val: i64) -> Result<i64, i64> {
    counter
        .value
        .compare_exchange(expected, new_val, Ordering::SeqCst, Ordering::SeqCst)
}

/// Returns true if the counter is zero.
#[allow(dead_code)]
pub fn counter_is_zero(counter: &AtomicCounter) -> bool {
    counter.value.load(Ordering::SeqCst) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_counter() {
        let c = new_atomic_counter();
        assert_eq!(counter_value(&c), 0);
    }

    #[test]
    fn test_increment() {
        let c = new_atomic_counter();
        assert_eq!(counter_increment(&c), 1);
        assert_eq!(counter_increment(&c), 2);
    }

    #[test]
    fn test_decrement() {
        let c = new_atomic_counter();
        counter_increment(&c);
        counter_increment(&c);
        assert_eq!(counter_decrement(&c), 1);
    }

    #[test]
    fn test_reset() {
        let c = new_atomic_counter();
        counter_increment(&c);
        counter_reset(&c);
        assert_eq!(counter_value(&c), 0);
    }

    #[test]
    fn test_add() {
        let c = new_atomic_counter();
        assert_eq!(counter_add(&c, 10), 10);
        assert_eq!(counter_value(&c), 10);
    }

    #[test]
    fn test_compare_exchange_success() {
        let c = new_atomic_counter();
        counter_add(&c, 5);
        let result = counter_compare_exchange(&c, 5, 10);
        assert_eq!(result, Ok(5));
        assert_eq!(counter_value(&c), 10);
    }

    #[test]
    fn test_compare_exchange_failure() {
        let c = new_atomic_counter();
        counter_add(&c, 5);
        let result = counter_compare_exchange(&c, 3, 10);
        assert!(result.is_err());
        assert_eq!(counter_value(&c), 5);
    }

    #[test]
    fn test_is_zero() {
        let c = new_atomic_counter();
        assert!(counter_is_zero(&c));
        counter_increment(&c);
        assert!(!counter_is_zero(&c));
    }

    #[test]
    fn test_negative() {
        let c = new_atomic_counter();
        counter_decrement(&c);
        assert_eq!(counter_value(&c), -1);
    }

    #[test]
    fn test_add_negative() {
        let c = new_atomic_counter();
        counter_add(&c, 10);
        counter_add(&c, -3);
        assert_eq!(counter_value(&c), 7);
    }
}
