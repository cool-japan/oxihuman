// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A table mapping levels (0..N) to threshold values and labels.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelTable {
    entries: Vec<LevelEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelEntry {
    pub level: u32,
    pub threshold: f64,
    pub label: String,
}

#[allow(dead_code)]
impl LevelTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_level(&mut self, level: u32, threshold: f64, label: &str) {
        self.entries.push(LevelEntry {
            level,
            threshold,
            label: label.to_string(),
        });
        self.entries.sort_by_key(|e| e.level);
    }

    pub fn level_for_value(&self, value: f64) -> Option<u32> {
        let mut best: Option<u32> = None;
        for entry in &self.entries {
            if value >= entry.threshold {
                best = Some(entry.level);
            }
        }
        best
    }

    pub fn label_for_level(&self, level: u32) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.level == level)
            .map(|e| e.label.as_str())
    }

    pub fn threshold_for_level(&self, level: u32) -> Option<f64> {
        self.entries
            .iter()
            .find(|e| e.level == level)
            .map(|e| e.threshold)
    }

    pub fn max_level(&self) -> Option<u32> {
        self.entries.last().map(|e| e.level)
    }

    pub fn num_levels(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn levels(&self) -> Vec<u32> {
        self.entries.iter().map(|e| e.level).collect()
    }

    pub fn label_for_value(&self, value: f64) -> Option<&str> {
        self.level_for_value(value)
            .and_then(|l| self.label_for_level(l))
    }
}

impl Default for LevelTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let t = LevelTable::new();
        assert!(t.is_empty());
    }

    #[test]
    fn test_add_level() {
        let mut t = LevelTable::new();
        t.add_level(0, 0.0, "beginner");
        t.add_level(1, 100.0, "intermediate");
        assert_eq!(t.num_levels(), 2);
    }

    #[test]
    fn test_level_for_value() {
        let mut t = LevelTable::new();
        t.add_level(0, 0.0, "low");
        t.add_level(1, 50.0, "mid");
        t.add_level(2, 100.0, "high");
        assert_eq!(t.level_for_value(75.0), Some(1));
        assert_eq!(t.level_for_value(100.0), Some(2));
        assert_eq!(t.level_for_value(-1.0), None);
    }

    #[test]
    fn test_label_for_level() {
        let mut t = LevelTable::new();
        t.add_level(0, 0.0, "zero");
        assert_eq!(t.label_for_level(0), Some("zero"));
        assert_eq!(t.label_for_level(99), None);
    }

    #[test]
    fn test_threshold_for_level() {
        let mut t = LevelTable::new();
        t.add_level(1, 42.0, "answer");
        assert!((t.threshold_for_level(1).expect("should succeed") - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_max_level() {
        let mut t = LevelTable::new();
        t.add_level(0, 0.0, "a");
        t.add_level(5, 500.0, "b");
        assert_eq!(t.max_level(), Some(5));
    }

    #[test]
    fn test_clear() {
        let mut t = LevelTable::new();
        t.add_level(0, 0.0, "x");
        t.clear();
        assert!(t.is_empty());
    }

    #[test]
    fn test_levels() {
        let mut t = LevelTable::new();
        t.add_level(2, 20.0, "b");
        t.add_level(0, 0.0, "a");
        let lvls = t.levels();
        assert_eq!(lvls, vec![0, 2]);
    }

    #[test]
    fn test_label_for_value() {
        let mut t = LevelTable::new();
        t.add_level(0, 0.0, "low");
        t.add_level(1, 50.0, "high");
        assert_eq!(t.label_for_value(25.0), Some("low"));
        assert_eq!(t.label_for_value(75.0), Some("high"));
    }

    #[test]
    fn test_empty_level_for_value() {
        let t = LevelTable::new();
        assert!(t.level_for_value(5.0).is_none());
    }
}
