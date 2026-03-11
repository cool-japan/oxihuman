// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Metric counter: named counters with min/max/sum tracking.

use std::collections::HashMap;

/// Per-metric statistics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MetricStats {
    pub name: String,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl MetricStats {
    fn new(name: &str) -> Self {
        MetricStats {
            name: name.to_string(),
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    fn record(&mut self, v: f64) {
        self.count += 1;
        self.sum += v;
        if v < self.min {
            self.min = v;
        }
        if v > self.max {
            self.max = v;
        }
    }

    /// Mean value.
    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.sum / self.count as f64
    }
}

/// Metric counter registry.
#[derive(Debug)]
#[allow(dead_code)]
pub struct MetricCounter {
    metrics: HashMap<String, MetricStats>,
}

/// Create a new MetricCounter.
#[allow(dead_code)]
pub fn new_metric_counter() -> MetricCounter {
    MetricCounter {
        metrics: HashMap::new(),
    }
}

/// Record a value for a named metric.
#[allow(dead_code)]
pub fn mc_record(mc: &mut MetricCounter, name: &str, value: f64) {
    mc.metrics
        .entry(name.to_string())
        .or_insert_with(|| MetricStats::new(name))
        .record(value);
}

/// Increment a counter metric by 1.
#[allow(dead_code)]
pub fn mc_increment(mc: &mut MetricCounter, name: &str) {
    mc_record(mc, name, 1.0);
}

/// Get stats for a metric.
#[allow(dead_code)]
pub fn mc_stats<'a>(mc: &'a MetricCounter, name: &str) -> Option<&'a MetricStats> {
    mc.metrics.get(name)
}

/// Mean of a metric.
#[allow(dead_code)]
pub fn mc_mean(mc: &MetricCounter, name: &str) -> f64 {
    mc.metrics.get(name).map(|s| s.mean()).unwrap_or(0.0)
}

/// Sum of a metric.
#[allow(dead_code)]
pub fn mc_sum(mc: &MetricCounter, name: &str) -> f64 {
    mc.metrics.get(name).map(|s| s.sum).unwrap_or(0.0)
}

/// Count of samples for a metric.
#[allow(dead_code)]
pub fn mc_count(mc: &MetricCounter, name: &str) -> u64 {
    mc.metrics.get(name).map(|s| s.count).unwrap_or(0)
}

/// Reset a specific metric.
#[allow(dead_code)]
pub fn mc_reset_one(mc: &mut MetricCounter, name: &str) {
    mc.metrics.remove(name);
}

/// Reset all metrics.
#[allow(dead_code)]
pub fn mc_reset_all(mc: &mut MetricCounter) {
    mc.metrics.clear();
}

/// All metric names.
#[allow(dead_code)]
pub fn mc_names(mc: &MetricCounter) -> Vec<String> {
    mc.metrics.keys().cloned().collect()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn mc_to_json(mc: &MetricCounter) -> String {
    let items: Vec<String> = mc
        .metrics
        .values()
        .map(|s| {
            format!(
                r#"{{"name":"{}","count":{},"sum":{},"mean":{}}}"#,
                s.name,
                s.count,
                s.sum,
                s.mean()
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_mean() {
        let mut mc = new_metric_counter();
        mc_record(&mut mc, "fps", 60.0);
        mc_record(&mut mc, "fps", 30.0);
        assert!((mc_mean(&mc, "fps") - 45.0).abs() < 1e-9);
    }

    #[test]
    fn test_increment() {
        let mut mc = new_metric_counter();
        mc_increment(&mut mc, "calls");
        mc_increment(&mut mc, "calls");
        assert_eq!(mc_count(&mc, "calls"), 2);
    }

    #[test]
    fn test_sum() {
        let mut mc = new_metric_counter();
        mc_record(&mut mc, "bytes", 100.0);
        mc_record(&mut mc, "bytes", 200.0);
        assert!((mc_sum(&mc, "bytes") - 300.0).abs() < 1e-9);
    }

    #[test]
    fn test_min_max() {
        let mut mc = new_metric_counter();
        mc_record(&mut mc, "lat", 5.0);
        mc_record(&mut mc, "lat", 15.0);
        mc_record(&mut mc, "lat", 10.0);
        let s = mc_stats(&mc, "lat").unwrap();
        assert!((s.min - 5.0).abs() < 1e-9);
        assert!((s.max - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_missing_metric() {
        let mc = new_metric_counter();
        assert_eq!(mc_count(&mc, "none"), 0);
        assert!(mc_stats(&mc, "none").is_none());
    }

    #[test]
    fn test_reset_one() {
        let mut mc = new_metric_counter();
        mc_increment(&mut mc, "x");
        mc_reset_one(&mut mc, "x");
        assert_eq!(mc_count(&mc, "x"), 0);
    }

    #[test]
    fn test_reset_all() {
        let mut mc = new_metric_counter();
        mc_increment(&mut mc, "a");
        mc_increment(&mut mc, "b");
        mc_reset_all(&mut mc);
        assert!(mc_names(&mc).is_empty());
    }

    #[test]
    fn test_json() {
        let mut mc = new_metric_counter();
        mc_increment(&mut mc, "ticks");
        let j = mc_to_json(&mc);
        assert!(j.contains("ticks"));
    }

    #[test]
    fn test_multiple_metrics() {
        let mut mc = new_metric_counter();
        mc_increment(&mut mc, "a");
        mc_increment(&mut mc, "b");
        assert_eq!(mc_names(&mc).len(), 2);
    }
}
