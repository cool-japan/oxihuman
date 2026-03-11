// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Persistent vector stub — uses path-copying to produce new versions on
//! each modification, keeping all prior versions intact.

/// A version snapshot of the persistent vector.
pub type PvecVersion = usize;

/// Persistent vector that retains all historical versions.
pub struct PersistentVector<T: Clone> {
    versions: Vec<Vec<T>>,
}

impl<T: Clone> PersistentVector<T> {
    /// Create an empty persistent vector.
    pub fn new() -> Self {
        Self {
            versions: vec![Vec::new()],
        }
    }

    /// Current (latest) version number.
    pub fn current_version(&self) -> PvecVersion {
        self.versions.len() - 1
    }

    /// Push `value` onto the current version, creating a new version.
    pub fn push(&mut self, value: T) -> PvecVersion {
        let mut next = self.versions.last().cloned().unwrap_or_default();
        next.push(value);
        self.versions.push(next);
        self.current_version()
    }

    /// Pop the last element of the current version, creating a new version.
    pub fn pop(&mut self) -> PvecVersion {
        let mut next = self.versions.last().cloned().unwrap_or_default();
        next.pop();
        self.versions.push(next);
        self.current_version()
    }

    /// Set element at `index` in the current version to `value`.
    pub fn set(&mut self, index: usize, value: T) -> Option<PvecVersion> {
        let len = self.versions.last().map_or(0, |v| v.len());
        if index >= len {
            return None;
        }
        let mut next = self.versions.last().cloned().unwrap_or_default();
        next[index] = value;
        self.versions.push(next);
        Some(self.current_version())
    }

    /// Get element at `index` in the current version.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.versions.last()?.get(index)
    }

    /// Get element at `index` in a specific `version`.
    pub fn get_at(&self, version: PvecVersion, index: usize) -> Option<&T> {
        self.versions.get(version)?.get(index)
    }

    /// Length of the current version.
    pub fn len(&self) -> usize {
        self.versions.last().map(|v| v.len()).unwrap_or(0)
    }

    /// True if current version is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total number of stored versions.
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

impl<T: Clone> Default for PersistentVector<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new empty persistent vector.
pub fn new_persistent_vector<T: Clone>() -> PersistentVector<T> {
    PersistentVector::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_get() {
        let mut v: PersistentVector<i32> = PersistentVector::new();
        v.push(10);
        assert_eq!(v.get(0), Some(&10)); /* element accessible */
    }

    #[test]
    fn test_old_version_preserved() {
        let mut v: PersistentVector<i32> = PersistentVector::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.get_at(1, 0), Some(&1)); /* version 1 has one element */
        assert_eq!(v.get_at(1, 1), None); /* version 1 lacks second */
    }

    #[test]
    fn test_pop_creates_version() {
        let mut v: PersistentVector<i32> = PersistentVector::new();
        v.push(1);
        let _vpush = v.push(2);
        let vpop = v.pop();
        assert_eq!(v.get_at(vpop, 1), None); /* element gone in new version */
    }

    #[test]
    fn test_set() {
        let mut v: PersistentVector<i32> = PersistentVector::new();
        v.push(1);
        let new_ver = v.set(0, 99).unwrap();
        assert_eq!(v.get_at(new_ver, 0), Some(&99)); /* updated */
        assert_eq!(v.get_at(1, 0), Some(&1)); /* old version unchanged */
    }

    #[test]
    fn test_set_out_of_bounds() {
        let mut v: PersistentVector<i32> = PersistentVector::new();
        assert!(v.set(5, 42).is_none()); /* out of bounds returns None */
    }

    #[test]
    fn test_len() {
        let mut v: PersistentVector<i32> = PersistentVector::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 2); /* correct length */
    }

    #[test]
    fn test_is_empty() {
        let v: PersistentVector<i32> = PersistentVector::new();
        assert!(v.is_empty()); /* initially empty */
    }

    #[test]
    fn test_version_count() {
        let mut v: PersistentVector<i32> = PersistentVector::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.version_count(), 3); /* initial + 2 pushes */
    }

    #[test]
    fn test_default() {
        let v: PersistentVector<i32> = PersistentVector::default();
        assert!(v.is_empty()); /* default is empty */
    }

    #[test]
    fn test_new_helper() {
        let v = new_persistent_vector::<u8>();
        assert!(v.is_empty()); /* helper works */
    }
}
