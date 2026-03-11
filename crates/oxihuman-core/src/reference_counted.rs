// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Manual reference-counted handle — a simplified, non-thread-safe Rc-like
//! wrapper implemented without the standard library Rc to illustrate the
//! mechanics of reference counting.

use std::cell::Cell;

struct RcInner<T> {
    value: T,
    strong: Cell<usize>,
}

/// A manually reference-counted handle.
pub struct RefCounted<T> {
    inner: *mut RcInner<T>,
}

impl<T> RefCounted<T> {
    /// Wrap a value in a reference-counted handle (count starts at 1).
    pub fn new(value: T) -> Self {
        let inner = Box::into_raw(Box::new(RcInner {
            value,
            strong: Cell::new(1),
        }));
        Self { inner }
    }

    /// Current strong reference count.
    pub fn ref_count(&self) -> usize {
        unsafe { (*self.inner).strong.get() }
    }

    /// Borrow the inner value.
    pub fn get(&self) -> &T {
        unsafe { &(*self.inner).value }
    }

    /// Clone this handle, incrementing the reference count.
    pub fn clone_handle(&self) -> Self {
        let count = unsafe { (*self.inner).strong.get() };
        unsafe { (*self.inner).strong.set(count + 1) };
        Self { inner: self.inner }
    }

    /// Returns true if this is the only handle owning the value.
    pub fn is_unique(&self) -> bool {
        self.ref_count() == 1
    }
}

impl<T> Drop for RefCounted<T> {
    fn drop(&mut self) {
        let count = unsafe { (*self.inner).strong.get() };
        if count == 1 {
            /* last reference — drop the box */
            unsafe { drop(Box::from_raw(self.inner)) };
        } else {
            unsafe { (*self.inner).strong.set(count - 1) };
        }
    }
}

/// Wrap a value in a reference-counted handle.
pub fn new_ref_counted<T>(value: T) -> RefCounted<T> {
    RefCounted::new(value)
}

/// Strong count of a handle.
pub fn rc_count<T>(handle: &RefCounted<T>) -> usize {
    handle.ref_count()
}

/// Check uniqueness of a handle.
pub fn rc_is_unique<T>(handle: &RefCounted<T>) -> bool {
    handle.is_unique()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_count() {
        let h = RefCounted::new(42);
        assert_eq!(h.ref_count(), 1); /* fresh handle has count 1 */
    }

    #[test]
    fn test_clone_increments() {
        let h = RefCounted::new(42);
        let h2 = h.clone_handle();
        assert_eq!(h.ref_count(), 2); /* clone increments */
        assert_eq!(h2.ref_count(), 2);
    }

    #[test]
    fn test_get() {
        let h = RefCounted::new(99);
        assert_eq!(*h.get(), 99); /* value accessible */
    }

    #[test]
    fn test_drop_decrements() {
        let h = RefCounted::new(1);
        {
            let _h2 = h.clone_handle();
            assert_eq!(h.ref_count(), 2); /* two refs */
        }
        assert_eq!(h.ref_count(), 1); /* back to one after drop */
    }

    #[test]
    fn test_is_unique_true() {
        let h = RefCounted::new(5);
        assert!(h.is_unique()); /* single owner */
    }

    #[test]
    fn test_is_unique_false() {
        let h = RefCounted::new(5);
        let _h2 = h.clone_handle();
        assert!(!h.is_unique()); /* two owners */
    }

    #[test]
    fn test_new_helper() {
        let h = new_ref_counted(7u32);
        assert_eq!(*h.get(), 7); /* helper works */
    }

    #[test]
    fn test_rc_count_helper() {
        let h = RefCounted::new(3);
        assert_eq!(rc_count(&h), 1); /* helper returns count */
    }

    #[test]
    fn test_rc_is_unique_helper() {
        let h = RefCounted::new(0);
        assert!(rc_is_unique(&h)); /* unique by default */
    }

    #[test]
    fn test_multiple_clones() {
        let h = RefCounted::new(0);
        let h2 = h.clone_handle();
        let h3 = h.clone_handle();
        assert_eq!(h.ref_count(), 3); /* three refs */
        drop(h2);
        drop(h3);
        assert_eq!(h.ref_count(), 1); /* back to one */
    }
}
