// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A weighted pool that maps items to weights for deterministic selection.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightedEntry<T> {
    pub item: T,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightedPool<T> {
    entries: Vec<WeightedEntry<T>>,
}

#[allow(dead_code)]
impl<T> WeightedPool<T> {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, item: T, weight: f32) {
        self.entries.push(WeightedEntry { item, weight: weight.max(0.0) });
    }

    pub fn total_weight(&self) -> f32 {
        self.entries.iter().map(|e| e.weight).sum()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Select by normalized position in [0, 1].
    pub fn select_by_normalized(&self, t: f32) -> Option<&T> {
        if self.entries.is_empty() {
            return None;
        }
        let total = self.total_weight();
        if total <= 0.0 {
            return self.entries.first().map(|e| &e.item);
        }
        let target = t.clamp(0.0, 1.0) * total;
        let mut acc = 0.0;
        for entry in &self.entries {
            acc += entry.weight;
            if acc >= target {
                return Some(&entry.item);
            }
        }
        self.entries.last().map(|e| &e.item)
    }

    pub fn normalize_weights(&mut self) {
        let total = self.total_weight();
        if total > 0.0 {
            for e in &mut self.entries {
                e.weight /= total;
            }
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<WeightedEntry<T>> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn weights(&self) -> Vec<f32> {
        self.entries.iter().map(|e| e.weight).collect()
    }

    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().map(|e| &e.item)
    }
}

impl<T> Default for WeightedPool<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_len() {
        let mut p = WeightedPool::new();
        p.add("a", 1.0);
        p.add("b", 2.0);
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn test_total_weight() {
        let mut p = WeightedPool::new();
        p.add(1, 3.0);
        p.add(2, 7.0);
        assert!((p.total_weight() - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_select_first() {
        let mut p = WeightedPool::new();
        p.add("a", 1.0);
        p.add("b", 1.0);
        assert_eq!(p.select_by_normalized(0.0), Some(&"a"));
    }

    #[test]
    fn test_select_last() {
        let mut p = WeightedPool::new();
        p.add("a", 1.0);
        p.add("b", 1.0);
        assert_eq!(p.select_by_normalized(1.0), Some(&"b"));
    }

    #[test]
    fn test_select_empty() {
        let p: WeightedPool<i32> = WeightedPool::new();
        assert_eq!(p.select_by_normalized(0.5), None);
    }

    #[test]
    fn test_normalize() {
        let mut p = WeightedPool::new();
        p.add(1, 2.0);
        p.add(2, 8.0);
        p.normalize_weights();
        assert!((p.total_weight() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_remove() {
        let mut p = WeightedPool::new();
        p.add(10, 1.0);
        p.add(20, 1.0);
        let removed = p.remove(0).unwrap();
        assert_eq!(removed.item, 10);
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut p = WeightedPool::new();
        p.add(1, 1.0);
        p.clear();
        assert!(p.is_empty());
    }

    #[test]
    fn test_weights() {
        let mut p = WeightedPool::new();
        p.add("x", 3.0);
        p.add("y", 5.0);
        assert_eq!(p.weights(), vec![3.0, 5.0]);
    }

    #[test]
    fn test_negative_weight_clamped() {
        let mut p = WeightedPool::new();
        p.add(1, -5.0);
        assert!((p.total_weight()).abs() < f32::EPSILON);
    }
}
