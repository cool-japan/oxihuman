// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Statistical aggregator over a stream of f64 values.

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Aggregator {
    pub values: Vec<f64>,
}

#[allow(dead_code)]
pub fn new_aggregator() -> Aggregator {
    Aggregator { values: Vec::new() }
}

#[allow(dead_code)]
pub fn push(agg: &mut Aggregator, v: f64) {
    agg.values.push(v);
}

#[allow(dead_code)]
pub fn mean(agg: &Aggregator) -> Option<f64> {
    if agg.values.is_empty() {
        return None;
    }
    Some(agg.values.iter().sum::<f64>() / agg.values.len() as f64)
}

#[allow(dead_code)]
pub fn variance(agg: &Aggregator) -> Option<f64> {
    let m = mean(agg)?;
    let var = agg.values.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / agg.values.len() as f64;
    Some(var)
}

#[allow(dead_code)]
pub fn std_dev(agg: &Aggregator) -> Option<f64> {
    variance(agg).map(|v| v.sqrt())
}

#[allow(dead_code)]
pub fn min_val(agg: &Aggregator) -> Option<f64> {
    agg.values.iter().copied().reduce(f64::min)
}

#[allow(dead_code)]
pub fn max_val(agg: &Aggregator) -> Option<f64> {
    agg.values.iter().copied().reduce(f64::max)
}

#[allow(dead_code)]
pub fn count(agg: &Aggregator) -> usize {
    agg.values.len()
}

#[allow(dead_code)]
pub fn clear(agg: &mut Aggregator) {
    agg.values.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let mut agg = new_aggregator();
        push(&mut agg, 1.0);
        push(&mut agg, 2.0);
        push(&mut agg, 3.0);
        assert!((mean(&agg).expect("should succeed") - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_variance() {
        let mut agg = new_aggregator();
        push(&mut agg, 2.0);
        push(&mut agg, 4.0);
        let var = variance(&agg).expect("should succeed");
        assert!((var - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_min() {
        let mut agg = new_aggregator();
        push(&mut agg, 5.0);
        push(&mut agg, 1.0);
        push(&mut agg, 3.0);
        assert!((min_val(&agg).expect("should succeed") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_max() {
        let mut agg = new_aggregator();
        push(&mut agg, 5.0);
        push(&mut agg, 1.0);
        push(&mut agg, 3.0);
        assert!((max_val(&agg).expect("should succeed") - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_empty_mean_returns_none() {
        let agg = new_aggregator();
        assert!(mean(&agg).is_none());
    }

    #[test]
    fn test_empty_variance_returns_none() {
        let agg = new_aggregator();
        assert!(variance(&agg).is_none());
    }

    #[test]
    fn test_count() {
        let mut agg = new_aggregator();
        push(&mut agg, 1.0);
        push(&mut agg, 2.0);
        assert_eq!(count(&agg), 2);
    }

    #[test]
    fn test_clear() {
        let mut agg = new_aggregator();
        push(&mut agg, 1.0);
        clear(&mut agg);
        assert_eq!(count(&agg), 0);
    }

    #[test]
    fn test_std_dev() {
        let mut agg = new_aggregator();
        push(&mut agg, 0.0);
        push(&mut agg, 2.0);
        let sd = std_dev(&agg).expect("should succeed");
        assert!((sd - 1.0).abs() < 1e-9);
    }
}
