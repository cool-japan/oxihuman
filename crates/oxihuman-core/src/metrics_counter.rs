// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Atomic metrics counter.

use std::collections::HashMap;

/// An atomic-style metrics counter (single-threaded stub).
#[derive(Debug, Default)]
pub struct MetricsCounter {
    counters: HashMap<String, u64>,
}

impl MetricsCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn increment_by(&mut self, name: &str, delta: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += delta;
    }

    pub fn reset(&mut self, name: &str) {
        self.counters.insert(name.to_string(), 0);
    }

    pub fn reset_all(&mut self) {
        for v in self.counters.values_mut() {
            *v = 0;
        }
    }

    pub fn get(&self, name: &str) -> u64 {
        *self.counters.get(name).unwrap_or(&0)
    }

    pub fn counter_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.counters.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn total(&self) -> u64 {
        self.counters.values().sum()
    }
}

pub fn new_metrics_counter() -> MetricsCounter {
    MetricsCounter::new()
}

pub fn mc_inc(counter: &mut MetricsCounter, name: &str) {
    counter.increment(name);
}

pub fn mc_inc_by(counter: &mut MetricsCounter, name: &str, delta: u64) {
    counter.increment_by(name, delta);
}

pub fn mc_get(counter: &MetricsCounter, name: &str) -> u64 {
    counter.get(name)
}

pub fn mc_reset(counter: &mut MetricsCounter, name: &str) {
    counter.reset(name);
}

pub fn mc_total(counter: &MetricsCounter) -> u64 {
    counter.total()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut c = new_metrics_counter();
        mc_inc(&mut c, "req");
        mc_inc(&mut c, "req");
        assert_eq!(mc_get(&c, "req"), 2);
    }

    #[test]
    fn test_increment_by() {
        let mut c = new_metrics_counter();
        mc_inc_by(&mut c, "bytes", 1024);
        assert_eq!(mc_get(&c, "bytes"), 1024);
    }

    #[test]
    fn test_unknown_counter_zero() {
        let c = new_metrics_counter();
        assert_eq!(mc_get(&c, "none"), 0);
    }

    #[test]
    fn test_reset_single() {
        let mut c = new_metrics_counter();
        mc_inc_by(&mut c, "x", 5);
        mc_reset(&mut c, "x");
        assert_eq!(mc_get(&c, "x"), 0);
    }

    #[test]
    fn test_reset_all() {
        let mut c = new_metrics_counter();
        mc_inc(&mut c, "a");
        mc_inc(&mut c, "b");
        c.reset_all();
        assert_eq!(mc_total(&c), 0);
    }

    #[test]
    fn test_total() {
        let mut c = new_metrics_counter();
        mc_inc_by(&mut c, "a", 3);
        mc_inc_by(&mut c, "b", 7);
        assert_eq!(mc_total(&c), 10);
    }

    #[test]
    fn test_counter_names_sorted() {
        let mut c = new_metrics_counter();
        mc_inc(&mut c, "z");
        mc_inc(&mut c, "a");
        let names = c.counter_names();
        assert_eq!(names[0], "a");
    }

    #[test]
    fn test_multiple_counters_independent() {
        let mut c = new_metrics_counter();
        mc_inc_by(&mut c, "x", 10);
        mc_inc_by(&mut c, "y", 20);
        assert_eq!(mc_get(&c, "x"), 10);
        assert_eq!(mc_get(&c, "y"), 20);
    }

    #[test]
    fn test_large_increment() {
        let mut c = new_metrics_counter();
        mc_inc_by(&mut c, "big", u64::MAX / 2);
        assert!(mc_get(&c, "big") > 0);
    }
}
