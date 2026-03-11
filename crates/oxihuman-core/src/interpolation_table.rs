// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lookup table with linear interpolation between samples.

/// A 1-D lookup table that maps evenly-spaced x values to y values
/// and linearly interpolates between adjacent entries.
#[derive(Debug, Clone)]
pub struct InterpolationTable {
    x_min: f64,
    x_max: f64,
    values: Vec<f64>,
}

impl InterpolationTable {
    /// Create a new table covering `[x_min, x_max]` with the given sample values.
    pub fn new(x_min: f64, x_max: f64, values: Vec<f64>) -> Self {
        assert!(values.len() >= 2, "need at least 2 samples");
        assert!(x_max > x_min, "x_max must be > x_min");
        InterpolationTable { x_min, x_max, values }
    }

    /// Number of sample points.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the table has no entries.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Look up the interpolated value at `x`.
    ///
    /// Clamps `x` to `[x_min, x_max]`.
    pub fn sample(&self, x: f64) -> f64 {
        let n = self.values.len();
        let t = ((x - self.x_min) / (self.x_max - self.x_min)).clamp(0.0, 1.0);
        let scaled = t * (n - 1) as f64;
        let lo = (scaled as usize).min(n - 2);
        let hi = lo + 1;
        let frac = scaled - lo as f64;
        self.values[lo] * (1.0 - frac) + self.values[hi] * frac
    }

    /// Set the value at sample index `i`.
    pub fn set_sample(&mut self, i: usize, v: f64) {
        self.values[i] = v;
    }

    /// Build a table from a closure evaluated at `n` evenly-spaced points.
    pub fn from_fn(x_min: f64, x_max: f64, n: usize, f: impl Fn(f64) -> f64) -> Self {
        assert!(n >= 2);
        let values = (0..n)
            .map(|i| {
                let x = x_min + (x_max - x_min) * i as f64 / (n - 1) as f64;
                f(x)
            })
            .collect();
        InterpolationTable { x_min, x_max, values }
    }
}

/// Create a new [`InterpolationTable`] from uniform samples.
pub fn new_interp_table(x_min: f64, x_max: f64, values: Vec<f64>) -> InterpolationTable {
    InterpolationTable::new(x_min, x_max, values)
}

/// Sample the table at `x`.
pub fn interp_table_sample(table: &InterpolationTable, x: f64) -> f64 {
    table.sample(x)
}

/// Number of entries in the table.
pub fn interp_table_len(table: &InterpolationTable) -> usize {
    table.len()
}

/// Build a table using a function.
pub fn interp_table_from_fn(
    x_min: f64,
    x_max: f64,
    n: usize,
    f: impl Fn(f64) -> f64,
) -> InterpolationTable {
    InterpolationTable::from_fn(x_min, x_max, n, f)
}

/// Returns the x range as a tuple.
pub fn interp_table_range(table: &InterpolationTable) -> (f64, f64) {
    (table.x_min, table.x_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_exact_endpoints() {
        let t = new_interp_table(0.0, 1.0, vec![0.0, 1.0]);
        assert!((t.sample(0.0) - 0.0).abs() < 1e-10 /* left endpoint */);
        assert!((t.sample(1.0) - 1.0).abs() < 1e-10 /* right endpoint */);
    }

    #[test]
    fn test_sample_midpoint() {
        let t = new_interp_table(0.0, 1.0, vec![0.0, 2.0]);
        assert!((t.sample(0.5) - 1.0).abs() < 1e-10 /* midpoint */);
    }

    #[test]
    fn test_clamp_below() {
        let t = new_interp_table(0.0, 1.0, vec![5.0, 10.0]);
        assert!((t.sample(-1.0) - 5.0).abs() < 1e-10 /* clamp low */);
    }

    #[test]
    fn test_clamp_above() {
        let t = new_interp_table(0.0, 1.0, vec![5.0, 10.0]);
        assert!((t.sample(2.0) - 10.0).abs() < 1e-10 /* clamp high */);
    }

    #[test]
    fn test_len() {
        let t = new_interp_table(0.0, 10.0, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(t.len(), 4 /* four samples */);
    }

    #[test]
    fn test_from_fn_identity() {
        let t = interp_table_from_fn(0.0, 1.0, 5, |x| x);
        assert!((t.sample(0.0) - 0.0).abs() < 1e-9 /* identity fn */);
        assert!((t.sample(1.0) - 1.0).abs() < 1e-9 /* identity fn */);
    }

    #[test]
    fn test_range() {
        let t = new_interp_table(2.0, 8.0, vec![0.0, 1.0]);
        let (lo, hi) = interp_table_range(&t);
        assert_eq!(lo, 2.0 /* x_min */);
        assert_eq!(hi, 8.0 /* x_max */);
    }

    #[test]
    fn test_three_segment_interp() {
        let t = new_interp_table(0.0, 2.0, vec![0.0, 1.0, 0.0]);
        /* peak at x=1 */
        assert!((t.sample(1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_is_not_empty() {
        let t = new_interp_table(0.0, 1.0, vec![0.0, 1.0]);
        assert!(!t.is_empty() /* non-empty table */);
    }

    #[test]
    fn test_set_sample() {
        let mut t = new_interp_table(0.0, 1.0, vec![0.0, 0.0]);
        t.set_sample(1, 5.0);
        assert!((t.sample(1.0) - 5.0).abs() < 1e-9 /* updated value */);
    }
}
