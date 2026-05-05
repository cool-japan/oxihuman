// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Thread-safe atomic counter backed by `AtomicI64`.

use std::sync::atomic::{AtomicI64, Ordering};

/// A thread-safe atomic counter.
pub struct AtomicCounter {
    value: AtomicI64,
}

/// Creates a new atomic counter starting at 0.
pub fn new_atomic_counter() -> AtomicCounter {
    AtomicCounter {
        value: AtomicI64::new(0),
    }
}

/// Creates a new atomic counter with the given initial value.
pub fn new_atomic_counter_with(initial: i64) -> AtomicCounter {
    AtomicCounter {
        value: AtomicI64::new(initial),
    }
}

/// Increments by 1 and returns the new value.
pub fn counter_increment(counter: &AtomicCounter) -> i64 {
    counter.value.fetch_add(1, Ordering::SeqCst) + 1
}

/// Decrements by 1 and returns the new value.
pub fn counter_decrement(counter: &AtomicCounter) -> i64 {
    counter.value.fetch_sub(1, Ordering::SeqCst) - 1
}

/// Returns the current value.  Alias: `counter_value`.
pub fn counter_get(counter: &AtomicCounter) -> i64 {
    counter.value.load(Ordering::SeqCst)
}

/// Returns the current value.
pub fn counter_value(counter: &AtomicCounter) -> i64 {
    counter.value.load(Ordering::SeqCst)
}

/// Resets to 0.
pub fn counter_reset(counter: &AtomicCounter) {
    counter.value.store(0, Ordering::SeqCst);
}

/// Adds `amount` and returns the new value.
pub fn counter_add(counter: &AtomicCounter, amount: i64) -> i64 {
    counter.value.fetch_add(amount, Ordering::SeqCst) + amount
}

/// If current == `expected`, atomically set to `new_val` and return `Ok(old)`.
/// Otherwise return `Err(current)`.
pub fn counter_compare_exchange(
    counter: &AtomicCounter,
    expected: i64,
    new_val: i64,
) -> Result<i64, i64> {
    counter
        .value
        .compare_exchange(expected, new_val, Ordering::SeqCst, Ordering::SeqCst)
}

/// Backward-compatible CAS: returns `true` iff the exchange succeeded.
pub fn counter_compare_and_swap(counter: &AtomicCounter, expected: i64, new_val: i64) -> bool {
    counter
        .value
        .compare_exchange(expected, new_val, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

/// Returns `true` if the counter is currently zero.
pub fn counter_is_zero(counter: &AtomicCounter) -> bool {
    counter.value.load(Ordering::SeqCst) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    // ---------- existing unit tests ----------

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

    #[test]
    fn test_new_counter_with_initial() {
        let c = new_atomic_counter_with(42);
        assert_eq!(counter_value(&c), 42);
    }

    #[test]
    fn test_counter_get_alias() {
        let c = new_atomic_counter_with(7);
        assert_eq!(counter_get(&c), 7);
    }

    #[test]
    fn test_compare_and_swap_success() {
        let c = new_atomic_counter_with(5);
        assert!(counter_compare_and_swap(&c, 5, 99));
        assert_eq!(counter_value(&c), 99);
    }

    #[test]
    fn test_compare_and_swap_failure() {
        let c = new_atomic_counter_with(5);
        assert!(!counter_compare_and_swap(&c, 0, 99));
        assert_eq!(counter_value(&c), 5);
    }

    // ---------- thread-safety test ----------

    #[test]
    fn test_thread_safety() {
        const THREADS: usize = 8;
        const INCREMENTS: usize = 10_000;

        let counter = Arc::new(new_atomic_counter());
        let mut handles = Vec::with_capacity(THREADS);

        for _ in 0..THREADS {
            let c = Arc::clone(&counter);
            handles.push(std::thread::spawn(move || {
                for _ in 0..INCREMENTS {
                    counter_increment(&c);
                }
            }));
        }

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        assert_eq!(counter_value(&counter), (THREADS * INCREMENTS) as i64);
    }

    // ---------- proptest: sum of deltas ----------

    proptest! {
        #[test]
        fn prop_counter_add_matches_wrapping_sum(
            deltas in proptest::collection::vec(-1_000_000i64..1_000_000i64, 0..200)
        ) {
            let c = new_atomic_counter();
            for &d in &deltas {
                counter_add(&c, d);
            }
            let expected: i64 = deltas.iter().fold(0i64, |acc, &d| acc.wrapping_add(d));
            prop_assert_eq!(counter_value(&c), expected);
        }
    }
}
