// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Copy-on-write wrapper stub — shares the inner value between handles until
//! a mutable write is requested, at which point the data is cloned so that
//! other readers remain unaffected.

use std::sync::Arc;

/// A copy-on-write wrapper that shares data until mutation is requested.
pub struct CowHandle<T: Clone> {
    inner: Arc<T>,
}

impl<T: Clone> CowHandle<T> {
    /// Create a new handle owning `value`.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    /// Borrow the inner value without cloning.
    pub fn read(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference, cloning the inner value if shared.
    pub fn write(&mut self) -> &mut T {
        Arc::make_mut(&mut self.inner)
    }

    /// Number of handles sharing this data.
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Returns true if the inner data is exclusively owned (no sharing).
    pub fn is_exclusive(&self) -> bool {
        Arc::strong_count(&self.inner) == 1
    }

    /// Clone the handle without cloning the data (shared reference).
    pub fn share(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Create a new copy-on-write handle.
pub fn new_cow_handle<T: Clone>(value: T) -> CowHandle<T> {
    CowHandle::new(value)
}

/// Share a handle without copying data.
pub fn cow_share<T: Clone>(h: &CowHandle<T>) -> CowHandle<T> {
    h.share()
}

/// Read the current value.
pub fn cow_read<T: Clone>(h: &CowHandle<T>) -> &T {
    h.read()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let h = CowHandle::new(42);
        assert_eq!(*h.read(), 42); /* value readable */
    }

    #[test]
    fn test_write_exclusive() {
        let mut h = CowHandle::new(1);
        *h.write() = 99;
        assert_eq!(*h.read(), 99); /* mutation applied */
    }

    #[test]
    fn test_share_increases_ref_count() {
        let h = CowHandle::new(0);
        let h2 = h.share();
        assert_eq!(h.ref_count(), 2); /* two handles share */
        drop(h2);
    }

    #[test]
    fn test_write_clones_shared() {
        let h = CowHandle::new(vec![1, 2, 3]);
        let h2 = h.share();
        /* h and h2 share; writing h should clone */
        let mut h_mut = h;
        h_mut.write().push(4);
        assert_eq!(h_mut.read().len(), 4); /* h_mut has the new element */
        assert_eq!(h2.read().len(), 3); /* h2 unaffected */
    }

    #[test]
    fn test_is_exclusive_initially() {
        let h = CowHandle::new(5);
        assert!(h.is_exclusive()); /* no sharing initially */
    }

    #[test]
    fn test_is_exclusive_after_share() {
        let h = CowHandle::new(5);
        let h2 = h.share();
        assert!(!h.is_exclusive()); /* shared */
        drop(h2);
    }

    #[test]
    fn test_is_exclusive_after_clone_drop() {
        let h = CowHandle::new(5);
        {
            let _h2 = h.share();
        }
        assert!(h.is_exclusive()); /* exclusive after other handle dropped */
    }

    #[test]
    fn test_new_helper() {
        let h = new_cow_handle(100u32);
        assert_eq!(*h.read(), 100); /* helper works */
    }

    #[test]
    fn test_cow_share_helper() {
        let h = new_cow_handle(7);
        let h2 = cow_share(&h);
        assert_eq!(*cow_read(&h2), 7); /* share and read helpers work */
    }

    #[test]
    fn test_ref_count_starts_at_one() {
        let h = CowHandle::new("hello");
        assert_eq!(h.ref_count(), 1); /* single owner */
    }
}
