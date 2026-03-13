// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! t-digest sketch for quantile approximation.

#![allow(dead_code)]

/// A centroid in the t-digest.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Centroid {
    pub mean: f64,
    pub weight: f64,
}

/// t-digest data structure for quantile estimation.
#[allow(dead_code)]
pub struct TDigest {
    centroids: Vec<Centroid>,
    compression: f64,
    total_weight: f64,
}

impl TDigest {
    #[allow(dead_code)]
    pub fn new(compression: f64) -> Self {
        Self {
            centroids: Vec::new(),
            compression: compression.max(10.0),
            total_weight: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, value: f64) {
        self.add_weighted(value, 1.0);
    }

    #[allow(dead_code)]
    pub fn add_weighted(&mut self, value: f64, weight: f64) {
        self.total_weight += weight;
        let idx = self.centroids.partition_point(|c| c.mean < value);

        let max_w = self.max_weight_for(idx);
        if let Some(c) = self.centroids.get_mut(idx) {
            if c.mean == value || c.weight + weight <= max_w {
                c.mean = (c.mean * c.weight + value * weight) / (c.weight + weight);
                c.weight += weight;
                return;
            }
        }
        self.centroids.insert(
            idx,
            Centroid {
                mean: value,
                weight,
            },
        );
        self.compress();
    }

    fn max_weight_for(&self, idx: usize) -> f64 {
        let q = if self.total_weight > 0.0 {
            let cumulative: f64 = self.centroids[..idx].iter().map(|c| c.weight).sum();
            (cumulative + self.centroids.get(idx).map_or(0.0, |c| c.weight / 2.0))
                / self.total_weight
        } else {
            0.5
        };
        4.0 * self.total_weight * q * (1.0 - q) / self.compression
    }

    fn compress(&mut self) {
        let max_centroids = (self.compression * std::f64::consts::PI / 2.0).ceil() as usize;
        if self.centroids.len() <= max_centroids {
            return;
        }
        /* Sort by mean before compressing */
        self.centroids.sort_by(|a, b| {
            a.mean
                .partial_cmp(&b.mean)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let total = self.total_weight;
        let mut merged: Vec<Centroid> = Vec::new();
        let mut cumulative_w = 0.0f64;
        for c in self.centroids.drain(..) {
            if let Some(last) = merged.last_mut() {
                /* Compute quantile at midpoint of last centroid */
                let q = (cumulative_w - last.weight / 2.0) / total;
                let q = q.clamp(0.0, 1.0);
                let limit = 4.0 * total * q * (1.0 - q) / self.compression;
                let limit = limit.max(1.0);
                if last.weight + c.weight <= limit {
                    last.mean =
                        (last.mean * last.weight + c.mean * c.weight) / (last.weight + c.weight);
                    last.weight += c.weight;
                    cumulative_w += c.weight;
                    continue;
                }
            }
            cumulative_w += c.weight;
            merged.push(c);
        }
        self.centroids = merged;
    }

    /// Estimate quantile q in [0, 1].
    #[allow(dead_code)]
    pub fn quantile(&self, q: f64) -> f64 {
        if self.centroids.is_empty() {
            return f64::NAN;
        }
        let target = q * self.total_weight;
        let mut cumulative = 0.0;
        for (i, c) in self.centroids.iter().enumerate() {
            let lower = cumulative;
            let upper = cumulative + c.weight;
            let mid = (lower + upper) / 2.0;
            if target <= mid {
                if i == 0 {
                    return c.mean;
                }
                let prev = &self.centroids[i - 1];
                let prev_mid = cumulative - prev.weight / 2.0;
                let frac = (target - prev_mid) / (mid - prev_mid);
                return prev.mean + frac * (c.mean - prev.mean);
            }
            cumulative += c.weight;
        }
        self.centroids.last().map_or(f64::NAN, |c| c.mean)
    }

    #[allow(dead_code)]
    pub fn count(&self) -> f64 {
        self.total_weight
    }

    #[allow(dead_code)]
    pub fn centroid_count(&self) -> usize {
        self.centroids.len()
    }

    #[allow(dead_code)]
    pub fn min(&self) -> f64 {
        self.centroids.first().map_or(f64::NAN, |c| c.mean)
    }

    #[allow(dead_code)]
    pub fn max(&self) -> f64 {
        self.centroids.last().map_or(f64::NAN, |c| c.mean)
    }

    #[allow(dead_code)]
    pub fn merge(&mut self, other: &TDigest) {
        for c in &other.centroids {
            self.add_weighted(c.mean, c.weight);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_quantile() {
        let td = TDigest::new(100.0);
        assert!(td.quantile(0.5).is_nan());
    }

    #[test]
    fn test_single_value() {
        let mut td = TDigest::new(100.0);
        td.add(5.0);
        assert!((td.quantile(0.5) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_median_uniform() {
        let mut td = TDigest::new(100.0);
        for i in 1..=100 {
            td.add(i as f64);
        }
        let m = td.quantile(0.5);
        assert!(m > 40.0 && m < 60.0, "median={m}");
    }

    #[test]
    fn test_min_max() {
        let mut td = TDigest::new(100.0);
        for i in [3.0, 1.0, 5.0, 2.0, 4.0] {
            td.add(i);
        }
        assert!((td.min() - 1.0).abs() < 1.0);
        assert!((td.max() - 5.0).abs() < 1.0);
    }

    #[test]
    fn test_count() {
        let mut td = TDigest::new(50.0);
        for _ in 0..10 {
            td.add(1.0);
        }
        assert!((td.count() - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_compression_clamp() {
        let td = TDigest::new(5.0);
        assert!(td.compression >= 10.0);
    }

    #[test]
    fn test_merge() {
        let mut td1 = TDigest::new(100.0);
        let mut td2 = TDigest::new(100.0);
        for i in 1..=50 {
            td1.add(i as f64);
        }
        for i in 51..=100 {
            td2.add(i as f64);
        }
        td1.merge(&td2);
        let m = td1.quantile(0.5);
        assert!(m > 40.0 && m < 60.0, "merged median={m}");
    }

    #[test]
    fn test_quantile_zero_one() {
        let mut td = TDigest::new(100.0);
        for i in 1..=10 {
            td.add(i as f64);
        }
        assert!(td.quantile(0.0) <= td.quantile(0.5));
        assert!(td.quantile(0.5) <= td.quantile(1.0));
    }

    #[test]
    fn test_centroid_count_bounded() {
        let mut td = TDigest::new(50.0);
        for i in 0..1000 {
            td.add(i as f64);
        }
        assert!(td.centroid_count() < 500);
    }

    #[test]
    fn test_add_weighted() {
        let mut td = TDigest::new(100.0);
        td.add_weighted(10.0, 5.0);
        assert!((td.count() - 5.0).abs() < 1e-6);
    }
}
