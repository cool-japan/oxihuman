// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Metrics gauge with min/max tracking.

use std::collections::HashMap;

/// A gauge entry tracking current, min, max values.
#[derive(Debug, Clone)]
pub struct GaugeEntry {
    pub current: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: u64,
}

impl GaugeEntry {
    fn new(initial: f64) -> Self {
        Self {
            current: initial,
            min: initial,
            max: initial,
            sample_count: 1,
        }
    }

    fn record(&mut self, value: f64) {
        self.current = value;
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sample_count += 1;
    }
}

/// Registry of named gauges.
#[derive(Debug, Default)]
pub struct MetricsGauge {
    gauges: HashMap<String, GaugeEntry>,
}

impl MetricsGauge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: &str, value: f64) {
        self.gauges
            .entry(name.to_string())
            .and_modify(|e| e.record(value))
            .or_insert_with(|| GaugeEntry::new(value));
    }

    pub fn get(&self, name: &str) -> Option<&GaugeEntry> {
        self.gauges.get(name)
    }

    pub fn current(&self, name: &str) -> f64 {
        self.gauges.get(name).map(|e| e.current).unwrap_or(0.0)
    }

    pub fn min_val(&self, name: &str) -> f64 {
        self.gauges.get(name).map(|e| e.min).unwrap_or(0.0)
    }

    pub fn max_val(&self, name: &str) -> f64 {
        self.gauges.get(name).map(|e| e.max).unwrap_or(0.0)
    }

    pub fn gauge_count(&self) -> usize {
        self.gauges.len()
    }

    pub fn reset(&mut self, name: &str) {
        self.gauges.remove(name);
    }
}

pub fn new_metrics_gauge() -> MetricsGauge {
    MetricsGauge::new()
}

pub fn gauge_set(g: &mut MetricsGauge, name: &str, value: f64) {
    g.set(name, value);
}

pub fn gauge_current(g: &MetricsGauge, name: &str) -> f64 {
    g.current(name)
}

pub fn gauge_min(g: &MetricsGauge, name: &str) -> f64 {
    g.min_val(name)
}

pub fn gauge_max(g: &MetricsGauge, name: &str) -> f64 {
    g.max_val(name)
}

pub fn gauge_count(g: &MetricsGauge) -> usize {
    g.gauge_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_current() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "cpu", 0.75);
        assert!((gauge_current(&g, "cpu") - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_min_tracking() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "temp", 20.0);
        gauge_set(&mut g, "temp", 15.0);
        gauge_set(&mut g, "temp", 25.0);
        assert!((gauge_min(&g, "temp") - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_max_tracking() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "mem", 100.0);
        gauge_set(&mut g, "mem", 200.0);
        gauge_set(&mut g, "mem", 150.0);
        assert!((gauge_max(&g, "mem") - 200.0).abs() < 1e-10);
    }

    #[test]
    fn test_sample_count() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "x", 1.0);
        gauge_set(&mut g, "x", 2.0);
        gauge_set(&mut g, "x", 3.0);
        assert_eq!(g.get("x").expect("should succeed").sample_count, 3);
    }

    #[test]
    fn test_unknown_returns_zero() {
        let g = new_metrics_gauge();
        assert_eq!(gauge_current(&g, "none"), 0.0);
        assert_eq!(gauge_min(&g, "none"), 0.0);
        assert_eq!(gauge_max(&g, "none"), 0.0);
    }

    #[test]
    fn test_gauge_count() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "a", 1.0);
        gauge_set(&mut g, "b", 2.0);
        assert_eq!(gauge_count(&g), 2);
    }

    #[test]
    fn test_reset() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "r", 5.0);
        g.reset("r");
        assert_eq!(gauge_count(&g), 0);
    }

    #[test]
    fn test_negative_values() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "delta", -10.0);
        gauge_set(&mut g, "delta", -5.0);
        assert!((gauge_min(&g, "delta") - (-10.0)).abs() < 1e-10);
    }

    #[test]
    fn test_first_value_is_min_and_max() {
        let mut g = new_metrics_gauge();
        gauge_set(&mut g, "single", 42.0);
        assert!((gauge_min(&g, "single") - 42.0).abs() < 1e-10);
        assert!((gauge_max(&g, "single") - 42.0).abs() < 1e-10);
    }
}
