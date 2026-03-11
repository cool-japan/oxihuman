// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A pool of named counting semaphores for concurrency-budget tracking.

use std::collections::HashMap;

/// A single counting semaphore.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Semaphore {
    pub name: String,
    pub capacity: u32,
    pub acquired: u32,
}

#[allow(dead_code)]
impl Semaphore {
    pub fn new(name: &str, capacity: u32) -> Self {
        Self {
            name: name.to_string(),
            capacity,
            acquired: 0,
        }
    }

    pub fn available(&self) -> u32 {
        self.capacity.saturating_sub(self.acquired)
    }

    pub fn try_acquire(&mut self, count: u32) -> bool {
        if self.available() >= count {
            self.acquired += count;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, count: u32) {
        self.acquired = self.acquired.saturating_sub(count);
    }

    pub fn is_full(&self) -> bool {
        self.acquired >= self.capacity
    }
}

/// Pool of named semaphores.
#[allow(dead_code)]
pub struct SemaphorePool {
    semaphores: HashMap<String, Semaphore>,
    total_acquired: u64,
    total_released: u64,
}

#[allow(dead_code)]
impl SemaphorePool {
    pub fn new() -> Self {
        Self {
            semaphores: HashMap::new(),
            total_acquired: 0,
            total_released: 0,
        }
    }

    pub fn add(&mut self, name: &str, capacity: u32) {
        self.semaphores
            .insert(name.to_string(), Semaphore::new(name, capacity));
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.semaphores.remove(name).is_some()
    }

    pub fn acquire(&mut self, name: &str, count: u32) -> bool {
        if let Some(s) = self.semaphores.get_mut(name) {
            if s.try_acquire(count) {
                self.total_acquired += u64::from(count);
                return true;
            }
        }
        false
    }

    pub fn release(&mut self, name: &str, count: u32) {
        if let Some(s) = self.semaphores.get_mut(name) {
            s.release(count);
            self.total_released += u64::from(count);
        }
    }

    pub fn available(&self, name: &str) -> u32 {
        self.semaphores
            .get(name)
            .map(|s| s.available())
            .unwrap_or(0)
    }

    pub fn capacity(&self, name: &str) -> u32 {
        self.semaphores.get(name).map(|s| s.capacity).unwrap_or(0)
    }

    pub fn count(&self) -> usize {
        self.semaphores.len()
    }

    pub fn total_acquired(&self) -> u64 {
        self.total_acquired
    }

    pub fn total_released(&self) -> u64 {
        self.total_released
    }

    pub fn names(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.semaphores.keys().map(|s| s.as_str()).collect();
        v.sort_unstable();
        v
    }

    pub fn is_full(&self, name: &str) -> bool {
        self.semaphores.get(name).is_some_and(|s| s.is_full())
    }

    pub fn clear(&mut self) {
        self.semaphores.clear();
    }
}

impl Default for SemaphorePool {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_semaphore_pool() -> SemaphorePool {
    SemaphorePool::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_and_release() {
        let mut p = new_semaphore_pool();
        p.add("net", 4);
        assert!(p.acquire("net", 2));
        assert_eq!(p.available("net"), 2);
        p.release("net", 2);
        assert_eq!(p.available("net"), 4);
    }

    #[test]
    fn over_capacity_fails() {
        let mut p = new_semaphore_pool();
        p.add("db", 2);
        assert!(p.acquire("db", 2));
        assert!(!p.acquire("db", 1));
    }

    #[test]
    fn unknown_semaphore() {
        let mut p = new_semaphore_pool();
        assert!(!p.acquire("missing", 1));
        assert_eq!(p.available("missing"), 0);
    }

    #[test]
    fn is_full() {
        let mut p = new_semaphore_pool();
        p.add("s", 1);
        p.acquire("s", 1);
        assert!(p.is_full("s"));
    }

    #[test]
    fn total_acquired_tracked() {
        let mut p = new_semaphore_pool();
        p.add("s", 10);
        p.acquire("s", 3);
        p.acquire("s", 2);
        assert_eq!(p.total_acquired(), 5);
    }

    #[test]
    fn remove_semaphore() {
        let mut p = new_semaphore_pool();
        p.add("x", 5);
        assert!(p.remove("x"));
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn names_sorted() {
        let mut p = new_semaphore_pool();
        p.add("b", 1);
        p.add("a", 1);
        assert_eq!(p.names(), vec!["a", "b"]);
    }

    #[test]
    fn capacity_query() {
        let mut p = new_semaphore_pool();
        p.add("q", 8);
        assert_eq!(p.capacity("q"), 8);
    }

    #[test]
    fn clear_empties_pool() {
        let mut p = new_semaphore_pool();
        p.add("a", 1);
        p.add("b", 2);
        p.clear();
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn release_no_underflow() {
        let mut p = new_semaphore_pool();
        p.add("s", 5);
        p.release("s", 100); // saturating
        assert_eq!(p.available("s"), 5);
    }
}
