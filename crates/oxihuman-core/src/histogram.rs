// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Value histogram / frequency counter.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistogramConfig {
    pub min: f32,
    pub max: f32,
    pub bins: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Histogram {
    pub config: HistogramConfig,
    pub counts: Vec<u64>,
    pub total: u64,
}

#[allow(dead_code)]
pub fn default_histogram_config() -> HistogramConfig {
    HistogramConfig { min: 0.0, max: 1.0, bins: 10 }
}

#[allow(dead_code)]
pub fn new_histogram(config: HistogramConfig) -> Histogram {
    let bins = config.bins.max(1);
    Histogram {
        counts: vec![0u64; bins],
        total: 0,
        config,
    }
}

#[allow(dead_code)]
pub fn hist_bin_for_value(h: &Histogram, value: f32) -> Option<usize> {
    let range = h.config.max - h.config.min;
    if range <= 0.0 || h.config.bins == 0 {
        return None;
    }
    if value < h.config.min || value > h.config.max {
        return None;
    }
    let frac = (value - h.config.min) / range;
    let bin = (frac * h.config.bins as f32).floor() as usize;
    Some(bin.min(h.config.bins - 1))
}

#[allow(dead_code)]
pub fn hist_record(h: &mut Histogram, value: f32) {
    if let Some(bin) = hist_bin_for_value(h, value) {
        h.counts[bin] += 1;
        h.total += 1;
    }
}

#[allow(dead_code)]
pub fn hist_count_at_bin(h: &Histogram, bin: usize) -> u64 {
    h.counts.get(bin).copied().unwrap_or(0)
}

#[allow(dead_code)]
pub fn hist_total(h: &Histogram) -> u64 {
    h.total
}

#[allow(dead_code)]
pub fn hist_max_count(h: &Histogram) -> u64 {
    h.counts.iter().copied().max().unwrap_or(0)
}

#[allow(dead_code)]
pub fn hist_clear(h: &mut Histogram) {
    for c in &mut h.counts {
        *c = 0;
    }
    h.total = 0;
}

#[allow(dead_code)]
pub fn hist_mean(h: &Histogram) -> f32 {
    if h.total == 0 {
        return 0.0;
    }
    let range = h.config.max - h.config.min;
    let bin_width = range / h.config.bins as f32;
    let mut sum = 0.0f64;
    for (i, &c) in h.counts.iter().enumerate() {
        let bin_center = h.config.min + (i as f32 + 0.5) * bin_width;
        sum += bin_center as f64 * c as f64;
    }
    (sum / h.total as f64) as f32
}

#[allow(dead_code)]
pub fn hist_to_json(h: &Histogram) -> String {
    let counts_str: Vec<String> = h.counts.iter().map(|c| c.to_string()).collect();
    format!(
        r#"{{"min":{},"max":{},"bins":{},"total":{},"counts":[{}]}}"#,
        h.config.min,
        h.config.max,
        h.config.bins,
        h.total,
        counts_str.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_histogram_empty() {
        let h = new_histogram(default_histogram_config());
        assert_eq!(hist_total(&h), 0);
    }

    #[test]
    fn test_record_and_total() {
        let mut h = new_histogram(default_histogram_config());
        hist_record(&mut h, 0.5);
        hist_record(&mut h, 0.1);
        assert_eq!(hist_total(&h), 2);
    }

    #[test]
    fn test_bin_for_value_in_range() {
        let h = new_histogram(default_histogram_config());
        let bin = hist_bin_for_value(&h, 0.0);
        assert_eq!(bin, Some(0));
    }

    #[test]
    fn test_bin_for_value_out_of_range() {
        let h = new_histogram(default_histogram_config());
        assert!(hist_bin_for_value(&h, -1.0).is_none());
        assert!(hist_bin_for_value(&h, 2.0).is_none());
    }

    #[test]
    fn test_count_at_bin() {
        let mut h = new_histogram(default_histogram_config());
        hist_record(&mut h, 0.05); // bin 0
        assert_eq!(hist_count_at_bin(&h, 0), 1);
    }

    #[test]
    fn test_max_count() {
        let mut h = new_histogram(default_histogram_config());
        hist_record(&mut h, 0.5);
        hist_record(&mut h, 0.5);
        hist_record(&mut h, 0.9);
        assert_eq!(hist_max_count(&h), 2);
    }

    #[test]
    fn test_clear() {
        let mut h = new_histogram(default_histogram_config());
        hist_record(&mut h, 0.3);
        hist_clear(&mut h);
        assert_eq!(hist_total(&h), 0);
        assert_eq!(hist_max_count(&h), 0);
    }

    #[test]
    fn test_mean_single_value() {
        let mut h = new_histogram(HistogramConfig { min: 0.0, max: 10.0, bins: 10 });
        hist_record(&mut h, 5.0); // bin 5, center ~5.5
        let m = hist_mean(&h);
        assert!(m > 0.0);
    }

    #[test]
    fn test_to_json_contains_counts() {
        let mut h = new_histogram(default_histogram_config());
        hist_record(&mut h, 0.5);
        let json = hist_to_json(&h);
        assert!(json.contains("counts"));
        assert!(json.contains("total"));
    }
}
