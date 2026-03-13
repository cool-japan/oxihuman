// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A fixed-size map backed by an array, indexed by enum variant ordinals.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnumMap<const N: usize> {
    values: [f64; N],
    labels: [&'static str; N],
}

#[allow(dead_code)]
impl<const N: usize> EnumMap<N> {
    pub fn new(labels: [&'static str; N]) -> Self {
        Self {
            values: [0.0; N],
            labels,
        }
    }

    pub fn set(&mut self, index: usize, value: f64) {
        if index < N {
            self.values[index] = value;
        }
    }

    pub fn get(&self, index: usize) -> f64 {
        if index < N {
            self.values[index]
        } else {
            0.0
        }
    }

    pub fn label(&self, index: usize) -> &'static str {
        if index < N {
            self.labels[index]
        } else {
            ""
        }
    }

    pub fn find_by_label(&self, label: &str) -> Option<usize> {
        self.labels.iter().position(|&l| l == label)
    }

    pub fn set_by_label(&mut self, label: &str, value: f64) -> bool {
        if let Some(idx) = self.find_by_label(label) {
            self.values[idx] = value;
            true
        } else {
            false
        }
    }

    pub fn get_by_label(&self, label: &str) -> Option<f64> {
        self.find_by_label(label).map(|idx| self.values[idx])
    }

    pub fn len(&self) -> usize {
        N
    }

    pub fn is_empty(&self) -> bool {
        N == 0
    }

    pub fn sum(&self) -> f64 {
        self.values.iter().sum()
    }

    #[allow(clippy::needless_range_loop)]
    pub fn reset(&mut self) {
        for i in 0..N {
            self.values[i] = 0.0;
        }
    }

    pub fn max_index(&self) -> Option<usize> {
        if N == 0 {
            return None;
        }
        let mut best = 0;
        #[allow(clippy::needless_range_loop)]
        for i in 1..N {
            if self.values[i] > self.values[best] {
                best = i;
            }
        }
        Some(best)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = EnumMap::<3>::new(["a", "b", "c"]);
        assert_eq!(m.len(), 3);
        assert!(!m.is_empty());
    }

    #[test]
    fn test_set_get() {
        let mut m = EnumMap::<2>::new(["x", "y"]);
        m.set(0, 1.5);
        assert!((m.get(0) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_label() {
        let m = EnumMap::<2>::new(["alpha", "beta"]);
        assert_eq!(m.label(0), "alpha");
        assert_eq!(m.label(1), "beta");
        assert_eq!(m.label(99), "");
    }

    #[test]
    fn test_find_by_label() {
        let m = EnumMap::<3>::new(["r", "g", "b"]);
        assert_eq!(m.find_by_label("g"), Some(1));
        assert_eq!(m.find_by_label("z"), None);
    }

    #[test]
    fn test_set_get_by_label() {
        let mut m = EnumMap::<2>::new(["w", "h"]);
        assert!(m.set_by_label("w", 10.0));
        assert!((m.get_by_label("w").expect("should succeed") - 10.0).abs() < 1e-9);
        assert!(!m.set_by_label("z", 1.0));
    }

    #[test]
    fn test_sum() {
        let mut m = EnumMap::<3>::new(["a", "b", "c"]);
        m.set(0, 1.0);
        m.set(1, 2.0);
        m.set(2, 3.0);
        assert!((m.sum() - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_reset() {
        let mut m = EnumMap::<2>::new(["a", "b"]);
        m.set(0, 5.0);
        m.reset();
        assert!((m.get(0)).abs() < 1e-9);
    }

    #[test]
    fn test_max_index() {
        let mut m = EnumMap::<3>::new(["a", "b", "c"]);
        m.set(0, 1.0);
        m.set(1, 5.0);
        m.set(2, 3.0);
        assert_eq!(m.max_index(), Some(1));
    }

    #[test]
    fn test_out_of_bounds() {
        let mut m = EnumMap::<2>::new(["a", "b"]);
        m.set(99, 1.0); // should not panic
        assert!((m.get(99)).abs() < 1e-9);
    }

    #[test]
    fn test_get_by_label_missing() {
        let m = EnumMap::<1>::new(["only"]);
        assert!(m.get_by_label("nope").is_none());
    }
}
