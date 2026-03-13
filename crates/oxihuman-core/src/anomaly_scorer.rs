// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Z-score anomaly detector.

#[derive(Debug, Clone)]
pub struct AnomalyScorer {
    mean: f64,
    variance: f64,
    count: u64,
    threshold: f64,
}

impl AnomalyScorer {
    pub fn new(threshold: f64) -> Self {
        AnomalyScorer {
            mean: 0.0,
            variance: 0.0,
            count: 0,
            threshold,
        }
    }

    /// Update online mean/variance using Welford's algorithm.
    pub fn update(&mut self, x: f64) {
        self.count += 1;
        let delta = x - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = x - self.mean;
        self.variance += delta * delta2;
    }

    pub fn std_dev(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        (self.variance / (self.count - 1) as f64).sqrt()
    }

    pub fn z_score(&self, x: f64) -> f64 {
        let std = self.std_dev();
        if std < f64::EPSILON {
            return 0.0;
        }
        (x - self.mean).abs() / std
    }

    pub fn is_anomaly(&self, x: f64) -> bool {
        self.z_score(x) > self.threshold
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn sample_count(&self) -> u64 {
        self.count
    }
}

pub fn z_score_batch(data: &[f64]) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let var = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    if std < f64::EPSILON {
        return vec![0.0; data.len()];
    }
    data.iter().map(|&x| (x - mean).abs() / std).collect()
}

pub fn flag_anomalies(data: &[f64], threshold: f64) -> Vec<bool> {
    let scores = z_score_batch(data);
    scores.iter().map(|&s| s > threshold).collect()
}

pub fn anomaly_count(data: &[f64], threshold: f64) -> usize {
    flag_anomalies(data, threshold)
        .iter()
        .filter(|&&b| b)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_online_mean() {
        let mut scorer = AnomalyScorer::new(3.0);
        scorer.update(1.0);
        scorer.update(2.0);
        scorer.update(3.0);
        assert!((scorer.mean() - 2.0).abs() < 1e-10, /* mean of 1,2,3 = 2 */);
    }

    #[test]
    fn test_std_dev_constant() {
        let mut scorer = AnomalyScorer::new(3.0);
        scorer.update(5.0);
        scorer.update(5.0);
        scorer.update(5.0);
        assert!(scorer.std_dev().abs() < 1e-10, /* constant series std = 0 */);
    }

    #[test]
    fn test_z_score_outlier() {
        let mut scorer = AnomalyScorer::new(3.0);
        for _ in 0..100 {
            scorer.update(0.0);
        }
        scorer.update(1.0);
        let z = scorer.z_score(1000.0);
        assert!(z > 3.0 /* extreme outlier has large z-score */,);
    }

    #[test]
    fn test_is_anomaly_true() {
        let mut scorer = AnomalyScorer::new(2.0);
        for _ in 0..50 {
            scorer.update(0.0);
        }
        scorer.update(10.0);
        assert!(scorer.is_anomaly(1000.0) /* huge value is anomaly */,);
    }

    #[test]
    fn test_z_score_batch_length() {
        let data = vec![1.0, 2.0, 3.0, 100.0];
        let scores = z_score_batch(&data);
        assert_eq!(scores.len(), 4);
    }

    #[test]
    fn test_z_score_batch_outlier() {
        let mut data = vec![0.0f64; 99];
        data.push(1000.0);
        let scores = z_score_batch(&data);
        let last = *scores.last().expect("should succeed");
        assert!(last > 3.0 /* 1000 is extreme outlier */,);
    }

    #[test]
    fn test_flag_anomalies() {
        let mut data = vec![0.0f64; 99];
        data.push(1000.0);
        let flags = flag_anomalies(&data, 3.0);
        assert!(*flags.last().expect("should succeed"), /* last value is flagged */);
    }

    #[test]
    fn test_anomaly_count() {
        let mut data = vec![0.0f64; 98];
        data.push(1000.0);
        data.push(-1000.0);
        let count = anomaly_count(&data, 3.0);
        assert_eq!(count, 2 /* two outliers */,);
    }

    #[test]
    fn test_empty_batch() {
        let scores = z_score_batch(&[]);
        assert!(scores.is_empty() /* empty input = empty output */,);
    }
}
