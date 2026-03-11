// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct AtomicCounter {
    pub value: i64,
}

pub fn new_atomic_counter(initial: i64) -> AtomicCounter {
    AtomicCounter { value: initial }
}

pub fn counter_increment(c: &mut AtomicCounter) -> i64 {
    c.value += 1;
    c.value
}

pub fn counter_decrement(c: &mut AtomicCounter) -> i64 {
    c.value -= 1;
    c.value
}

pub fn counter_add(c: &mut AtomicCounter, n: i64) -> i64 {
    c.value += n;
    c.value
}

pub fn counter_get(c: &AtomicCounter) -> i64 {
    c.value
}

pub fn counter_reset(c: &mut AtomicCounter) {
    c.value = 0;
}

pub fn counter_compare_and_swap(c: &mut AtomicCounter, expected: i64, new_val: i64) -> bool {
    if c.value == expected {
        c.value = new_val;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_value() {
        /* counter starts at provided initial value */
        let c = new_atomic_counter(10);
        assert_eq!(counter_get(&c), 10);
    }

    #[test]
    fn increment() {
        /* increment returns new value */
        let mut c = new_atomic_counter(0);
        assert_eq!(counter_increment(&mut c), 1);
    }

    #[test]
    fn decrement() {
        /* decrement returns new value */
        let mut c = new_atomic_counter(5);
        assert_eq!(counter_decrement(&mut c), 4);
    }

    #[test]
    fn add_positive() {
        /* add increments by n */
        let mut c = new_atomic_counter(0);
        assert_eq!(counter_add(&mut c, 10), 10);
    }

    #[test]
    fn reset() {
        /* reset sets value to zero */
        let mut c = new_atomic_counter(99);
        counter_reset(&mut c);
        assert_eq!(counter_get(&c), 0);
    }

    #[test]
    fn cas_success() {
        /* CAS succeeds when expected matches */
        let mut c = new_atomic_counter(5);
        assert!(counter_compare_and_swap(&mut c, 5, 10));
        assert_eq!(counter_get(&c), 10);
    }

    #[test]
    fn cas_failure() {
        /* CAS fails when expected does not match */
        let mut c = new_atomic_counter(5);
        assert!(!counter_compare_and_swap(&mut c, 99, 10));
        assert_eq!(counter_get(&c), 5);
    }
}
