// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A simple reference-counted handle wrapping data of type `T`.
#[allow(dead_code)]
pub struct RefCounted<T: Clone> {
    pub data: T,
    pub count: usize,
}

/// Create a new `RefCounted<T>` with an initial count of 1.
#[allow(dead_code)]
pub fn new_ref_counted<T: Clone>(data: T) -> RefCounted<T> {
    RefCounted { data, count: 1 }
}

/// Clone the data and increment the reference count. Returns the cloned data.
#[allow(dead_code)]
pub fn ref_counted_clone(rc: &mut RefCounted<String>) -> String {
    rc.count += 1;
    rc.data.clone()
}

/// Decrement the reference count (minimum 0).
#[allow(dead_code)]
pub fn ref_drop(rc: &mut RefCounted<String>) {
    if rc.count > 0 {
        rc.count -= 1;
    }
}

/// Return the current reference count.
#[allow(dead_code)]
pub fn ref_count(rc: &RefCounted<String>) -> usize {
    rc.count
}

/// Return a reference to the inner data.
#[allow(dead_code)]
pub fn ref_data(rc: &RefCounted<String>) -> &String {
    &rc.data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_count_is_one() {
        let rc = new_ref_counted("hello".to_string());
        assert_eq!(ref_count(&rc), 1);
    }

    #[test]
    fn data_is_stored() {
        let rc = new_ref_counted("world".to_string());
        assert_eq!(ref_data(&rc), "world");
    }

    #[test]
    fn clone_increments_count() {
        let mut rc = new_ref_counted("abc".to_string());
        let _ = ref_counted_clone(&mut rc);
        assert_eq!(ref_count(&rc), 2);
    }

    #[test]
    fn clone_returns_data() {
        let mut rc = new_ref_counted("data".to_string());
        let cloned = ref_counted_clone(&mut rc);
        assert_eq!(cloned, "data");
    }

    #[test]
    fn drop_decrements_count() {
        let mut rc = new_ref_counted("val".to_string());
        ref_counted_clone(&mut rc);
        assert_eq!(ref_count(&rc), 2);
        ref_drop(&mut rc);
        assert_eq!(ref_count(&rc), 1);
    }

    #[test]
    fn drop_does_not_go_below_zero() {
        let mut rc = new_ref_counted("x".to_string());
        ref_drop(&mut rc);
        ref_drop(&mut rc);
        assert_eq!(ref_count(&rc), 0);
    }

    #[test]
    fn multiple_clones_accumulate() {
        let mut rc = new_ref_counted("multi".to_string());
        for _ in 0..4 {
            ref_counted_clone(&mut rc);
        }
        assert_eq!(ref_count(&rc), 5);
    }

    #[test]
    fn drop_after_multiple_clones() {
        let mut rc = new_ref_counted("m".to_string());
        ref_counted_clone(&mut rc);
        ref_counted_clone(&mut rc);
        ref_drop(&mut rc);
        assert_eq!(ref_count(&rc), 2);
    }

    #[test]
    fn data_unchanged_after_clone_and_drop() {
        let mut rc = new_ref_counted("stable".to_string());
        ref_counted_clone(&mut rc);
        ref_drop(&mut rc);
        assert_eq!(ref_data(&rc), "stable");
    }

    #[test]
    fn new_ref_counted_generic_i32() {
        let rc: RefCounted<i32> = new_ref_counted(42);
        assert_eq!(rc.data, 42);
        assert_eq!(rc.count, 1);
    }
}
