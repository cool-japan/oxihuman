// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A single-assignment cell that lazily initializes a value on first access.

#[allow(dead_code)]
#[derive(Debug)]
pub struct LazyInit<T> {
    value: Option<T>,
}

#[allow(dead_code)]
impl<T> LazyInit<T> {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn with_value(value: T) -> Self {
        Self { value: Some(value) }
    }

    pub fn get_or_init(&mut self, f: impl FnOnce() -> T) -> &T {
        self.value.get_or_insert_with(f)
    }

    pub fn get_or_init_mut(&mut self, f: impl FnOnce() -> T) -> &mut T {
        self.value.get_or_insert_with(f)
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }

    pub fn is_initialized(&self) -> bool {
        self.value.is_some()
    }

    pub fn set(&mut self, value: T) -> bool {
        if self.value.is_some() {
            return false;
        }
        self.value = Some(value);
        true
    }

    pub fn take(&mut self) -> Option<T> {
        self.value.take()
    }

    pub fn reset(&mut self) {
        self.value = None;
    }

    pub fn into_inner(self) -> Option<T> {
        self.value
    }
}

impl<T> Default for LazyInit<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_init() {
        let mut lazy = LazyInit::new();
        let v = lazy.get_or_init(|| 42);
        assert_eq!(*v, 42);
    }

    #[test]
    fn test_init_once() {
        let mut lazy = LazyInit::new();
        lazy.get_or_init(|| 1);
        let v = lazy.get_or_init(|| 2);
        assert_eq!(*v, 1);
    }

    #[test]
    fn test_is_initialized() {
        let mut lazy: LazyInit<i32> = LazyInit::new();
        assert!(!lazy.is_initialized());
        lazy.get_or_init(|| 0);
        assert!(lazy.is_initialized());
    }

    #[test]
    fn test_get_before_init() {
        let lazy: LazyInit<i32> = LazyInit::new();
        assert!(lazy.get().is_none());
    }

    #[test]
    fn test_set() {
        let mut lazy = LazyInit::new();
        assert!(lazy.set(10));
        assert!(!lazy.set(20));
        assert_eq!(lazy.get(), Some(&10));
    }

    #[test]
    fn test_take() {
        let mut lazy = LazyInit::with_value(99);
        assert_eq!(lazy.take(), Some(99));
        assert!(!lazy.is_initialized());
    }

    #[test]
    fn test_reset() {
        let mut lazy = LazyInit::with_value(5);
        lazy.reset();
        assert!(lazy.get().is_none());
    }

    #[test]
    fn test_with_value() {
        let lazy = LazyInit::with_value(7);
        assert_eq!(lazy.get(), Some(&7));
    }

    #[test]
    fn test_into_inner() {
        let lazy = LazyInit::with_value(100);
        assert_eq!(lazy.into_inner(), Some(100));
    }

    #[test]
    fn test_get_or_init_mut() {
        let mut lazy = LazyInit::new();
        let v = lazy.get_or_init_mut(|| 10);
        *v = 20;
        assert_eq!(lazy.get(), Some(&20));
    }
}
