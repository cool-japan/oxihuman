// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Performance metrics tracking.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

impl MetricKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricKind::Counter => "counter",
            MetricKind::Gauge => "gauge",
            MetricKind::Histogram => "histogram",
            MetricKind::Timer => "timer",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetricSample {
    pub value: f64,
    pub timestamp_ms: u64,
}

impl MetricSample {
    pub fn new(value: f64, timestamp_ms: u64) -> Self {
        Self {
            value,
            timestamp_ms,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub kind: MetricKind,
    pub samples: Vec<MetricSample>,
    pub unit: String,
}

impl Metric {
    pub fn new(name: &str, kind: MetricKind, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            kind,
            samples: Vec::new(),
            unit: unit.to_string(),
        }
    }

    /// Mean value of all samples; None if empty.
    pub fn mean(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let sum: f64 = self.samples.iter().map(|s| s.value).sum();
        Some(sum / self.samples.len() as f64)
    }

    /// Last recorded value; None if empty.
    pub fn last(&self) -> Option<f64> {
        self.samples.last().map(|s| s.value)
    }

    pub fn to_json(&self) -> String {
        let samples: Vec<String> = self
            .samples
            .iter()
            .map(|s| format!(r#"{{"v":{:.6},"ts":{}}}"#, s.value, s.timestamp_ms))
            .collect();
        format!(
            r#"{{"name":"{}","kind":"{}","unit":"{}","samples":[{}]}}"#,
            self.name,
            self.kind.as_str(),
            self.unit,
            samples.join(",")
        )
    }
}

/// A simple timestamp-free clock substitute for tests (monotonic counter).
fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetricsRegistry {
    pub metrics: Vec<Metric>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    /// Find or create a metric entry.
    fn get_or_create(&mut self, name: &str, kind: MetricKind, unit: &str) -> &mut Metric {
        if let Some(pos) = self.metrics.iter().position(|m| m.name == name) {
            return &mut self.metrics[pos];
        }
        self.metrics.push(Metric::new(name, kind, unit));
        let len = self.metrics.len();
        &mut self.metrics[len - 1]
    }

    /// Record a counter increment.
    pub fn record_counter(&mut self, name: &str, value: f64) {
        let ts = now_ms();
        let metric = self.get_or_create(name, MetricKind::Counter, "count");
        // For counters accumulate the value.
        let prev = metric.last().unwrap_or(0.0);
        metric.samples.push(MetricSample::new(prev + value, ts));
    }

    /// Record a gauge (set to exact value).
    pub fn record_gauge(&mut self, name: &str, value: f64) {
        let ts = now_ms();
        let metric = self.get_or_create(name, MetricKind::Gauge, "");
        metric.samples.push(MetricSample::new(value, ts));
    }

    /// Add a histogram sample.
    pub fn record_histogram(&mut self, name: &str, value: f64) {
        let ts = now_ms();
        let metric = self.get_or_create(name, MetricKind::Histogram, "");
        metric.samples.push(MetricSample::new(value, ts));
    }

    /// Find a metric by name.
    pub fn find(&self, name: &str) -> Option<&Metric> {
        self.metrics.iter().find(|m| m.name == name)
    }

    /// Number of samples for a named metric.
    pub fn sample_count(&self, name: &str) -> usize {
        self.find(name).map(|m| m.samples.len()).unwrap_or(0)
    }

    /// Last recorded value for a named metric.
    pub fn last_value(&self, name: &str) -> Option<f64> {
        self.find(name)?.last()
    }

    /// Mean of all samples for a named metric.
    pub fn mean_value(&self, name: &str) -> Option<f64> {
        self.find(name)?.mean()
    }

    /// Clear all metrics and their samples.
    pub fn reset(&mut self) {
        self.metrics.clear();
    }

    /// JSON export of the full registry.
    pub fn to_json(&self) -> String {
        let ms: Vec<String> = self.metrics.iter().map(|m| m.to_json()).collect();
        format!(r#"{{"metrics":[{}]}}"#, ms.join(","))
    }

    /// List of all metric names.
    pub fn metric_names(&self) -> Vec<&str> {
        self.metrics.iter().map(|m| m.name.as_str()).collect()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_counter_creates_metric() {
        let mut r = MetricsRegistry::new();
        r.record_counter("frames", 1.0);
        assert!(r.find("frames").is_some());
    }

    #[test]
    fn record_counter_accumulates() {
        let mut r = MetricsRegistry::new();
        r.record_counter("frames", 1.0);
        r.record_counter("frames", 1.0);
        // second sample = 1+1 = 2
        assert!((r.last_value("frames").expect("should succeed") - 2.0).abs() < 1e-9);
    }

    #[test]
    fn record_gauge_sets_last_value() {
        let mut r = MetricsRegistry::new();
        r.record_gauge("fps", 60.0);
        r.record_gauge("fps", 30.0);
        assert!((r.last_value("fps").expect("should succeed") - 30.0).abs() < 1e-9);
    }

    #[test]
    fn record_histogram_increases_sample_count() {
        let mut r = MetricsRegistry::new();
        r.record_histogram("frame_time", 16.7);
        r.record_histogram("frame_time", 17.0);
        r.record_histogram("frame_time", 15.5);
        assert_eq!(r.sample_count("frame_time"), 3);
    }

    #[test]
    fn find_missing_returns_none() {
        let r = MetricsRegistry::new();
        assert!(r.find("nonexistent").is_none());
    }

    #[test]
    fn last_value_none_for_missing() {
        let r = MetricsRegistry::new();
        assert!(r.last_value("none").is_none());
    }

    #[test]
    fn mean_value_correct() {
        let mut r = MetricsRegistry::new();
        r.record_histogram("latency", 10.0);
        r.record_histogram("latency", 20.0);
        r.record_histogram("latency", 30.0);
        let mean = r.mean_value("latency").expect("should succeed");
        assert!((mean - 20.0).abs() < 1e-9);
    }

    #[test]
    fn mean_value_none_for_missing() {
        let r = MetricsRegistry::new();
        assert!(r.mean_value("nope").is_none());
    }

    #[test]
    fn reset_clears_all() {
        let mut r = MetricsRegistry::new();
        r.record_gauge("fps", 60.0);
        r.record_counter("frames", 1.0);
        r.reset();
        assert_eq!(r.metrics.len(), 0);
    }

    #[test]
    fn to_json_non_empty() {
        let mut r = MetricsRegistry::new();
        r.record_gauge("fps", 60.0);
        let j = r.to_json();
        assert!(!j.is_empty());
        assert!(j.contains("fps"));
    }

    #[test]
    fn metric_names_lists_all() {
        let mut r = MetricsRegistry::new();
        r.record_gauge("fps", 60.0);
        r.record_counter("frames", 1.0);
        let names = r.metric_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"fps"));
        assert!(names.contains(&"frames"));
    }

    #[test]
    fn sample_count_zero_for_missing() {
        let r = MetricsRegistry::new();
        assert_eq!(r.sample_count("unknown"), 0);
    }

    #[test]
    fn multiple_metric_kinds_coexist() {
        let mut r = MetricsRegistry::new();
        r.record_counter("c", 1.0);
        r.record_gauge("g", 5.0);
        r.record_histogram("h", 3.0);
        assert_eq!(r.metrics.len(), 3);
    }

    #[test]
    fn metric_kind_as_str() {
        assert_eq!(MetricKind::Counter.as_str(), "counter");
        assert_eq!(MetricKind::Gauge.as_str(), "gauge");
        assert_eq!(MetricKind::Histogram.as_str(), "histogram");
        assert_eq!(MetricKind::Timer.as_str(), "timer");
    }
}
