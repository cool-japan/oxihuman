// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Weak reference stub — tracks whether an associated strong handle is still
//! alive using a shared liveness flag, without prolonging the object's
//! lifetime.

use std::cell::Cell;
use std::rc::Rc;

/// Shared liveness token.
struct Token {
    alive: Cell<bool>,
}

/// A strong owner that keeps the liveness token set.
pub struct StrongOwner<T> {
    value: T,
    token: Rc<Token>,
}

impl<T> StrongOwner<T> {
    /// Create a new strong owner.
    pub fn new(value: T) -> (Self, WeakRef<T>) {
        let token = Rc::new(Token {
            alive: Cell::new(true),
        });
        let weak = WeakRef {
            token: Rc::clone(&token),
            _marker: std::marker::PhantomData,
        };
        (Self { value, token }, weak)
    }

    /// Access the inner value.
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T> Drop for StrongOwner<T> {
    fn drop(&mut self) {
        self.token.alive.set(false);
    }
}

/// A weak reference that cannot prevent the owner from dropping.
pub struct WeakRef<T> {
    token: Rc<Token>,
    _marker: std::marker::PhantomData<*const T>,
}

impl<T> WeakRef<T> {
    /// Returns true if the associated strong owner is still alive.
    pub fn is_alive(&self) -> bool {
        self.token.alive.get()
    }

    /// Returns false if the strong owner has been dropped.
    pub fn is_dangling(&self) -> bool {
        !self.is_alive()
    }

    /// Count of alive references to the token.
    pub fn token_ref_count(&self) -> usize {
        Rc::strong_count(&self.token)
    }
}

/// Create a strong owner and its weak reference.
pub fn new_weak_pair<T>(value: T) -> (StrongOwner<T>, WeakRef<T>) {
    StrongOwner::new(value)
}

/// Check whether a weak reference is still alive.
pub fn weak_is_alive<T>(w: &WeakRef<T>) -> bool {
    w.is_alive()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alive_while_owner_exists() {
        let (owner, weak) = StrongOwner::new(42);
        assert!(weak.is_alive()); /* alive while owner lives */
        drop(owner);
    }

    #[test]
    fn test_dangling_after_owner_drop() {
        let (owner, weak) = StrongOwner::new(42);
        drop(owner);
        assert!(weak.is_dangling()); /* dangling after drop */
    }

    #[test]
    fn test_get() {
        let (owner, _weak) = StrongOwner::new(99);
        assert_eq!(*owner.get(), 99); /* value accessible */
    }

    #[test]
    fn test_is_alive_helper() {
        let (_owner, weak) = StrongOwner::new(0);
        assert!(weak_is_alive(&weak)); /* helper works */
    }

    #[test]
    fn test_new_pair_helper() {
        let (_owner, weak) = new_weak_pair(1u32);
        assert!(weak.is_alive()); /* pair helper works */
    }

    #[test]
    fn test_multiple_weakrefs_not_supported_directly() {
        /* WeakRef is not Clone; only one per owner, test owner drop */
        let (owner, weak) = StrongOwner::new("hello");
        assert!(weak.is_alive());
        drop(owner);
        assert!(!weak.is_alive()); /* confirmed dead */
    }

    #[test]
    fn test_token_ref_count_alive() {
        let (_owner, weak) = StrongOwner::new(5);
        /* owner holds one Rc, weak holds one */
        assert_eq!(weak.token_ref_count(), 2); /* two refs to token */
    }

    #[test]
    fn test_token_ref_count_dead() {
        let (owner, weak) = StrongOwner::new(5);
        drop(owner);
        assert_eq!(weak.token_ref_count(), 1); /* only weak left */
    }

    #[test]
    fn test_is_not_dangling_when_alive() {
        let (_owner, weak) = StrongOwner::new(0);
        assert!(!weak.is_dangling()); /* not dangling while alive */
    }

    #[test]
    fn test_string_value() {
        let (owner, weak) = StrongOwner::new(String::from("hello"));
        assert_eq!(owner.get(), "hello");
        drop(owner);
        assert!(weak.is_dangling()); /* dangling after drop */
    }
}
