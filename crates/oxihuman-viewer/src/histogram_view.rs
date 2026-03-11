// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Histogram visualization for vertex attribute distribution.

/// Configuration for building a histogram.
#[allow(dead_code)]
pub struct HistogramConfig {
    pub bin_count: usize,
    pub range_min: f32,
    pub range_max: f32,
    pub title: String,
}

/// A single histogram bin.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct HistogramBin {
    pub low: f32,
    pub high: f32,
    pub count: u32,
}

/// A complete histogram.
#[allow(dead_code)]
pub struct Histogram {
    pub bins: Vec<HistogramBin>,
    pub total_samples: u32,
    pub title: String,
}

/// Type alias for entropy result.
#[allow(dead_code)]
pub type EntropyResult = f64;

// ── Public API ────────────────────────────────────────────────────────────────

/// Create a default histogram configuration with 32 bins over `[0, 1]`.
#[allow(dead_code)]
pub fn default_histogram_config() -> HistogramConfig {
    HistogramConfig {
        bin_count: 32,
        range_min: 0.0,
        range_max: 1.0,
        title: "Histogram".to_string(),
    }
}

/// Build a histogram from a slice of `f32` values.
#[allow(dead_code)]
pub fn build_histogram(data: &[f32], config: &HistogramConfig) -> Histogram {
    let n = config.bin_count.max(1);
    let range = config.range_max - config.range_min;
    let bin_width = if range.abs() < 1e-12 {
        1.0
    } else {
        range / n as f32
    };

    let mut bins: Vec<HistogramBin> = (0..n)
        .map(|i| {
            let low = config.range_min + i as f32 * bin_width;
            let high = low + bin_width;
            HistogramBin {
                low,
                high,
                count: 0,
            }
        })
        .collect();

    let mut total = 0u32;
    for &v in data {
        if v < config.range_min || v > config.range_max {
            continue;
        }
        let idx = if range.abs() < 1e-12 {
            0
        } else {
            let raw = ((v - config.range_min) / bin_width) as usize;
            raw.min(n - 1)
        };
        bins[idx].count += 1;
        total += 1;
    }

    Histogram {
        bins,
        total_samples: total,
        title: config.title.clone(),
    }
}

/// Return the number of bins.
#[allow(dead_code)]
pub fn histogram_bin_count(hist: &Histogram) -> usize {
    hist.bins.len()
}

/// Return the maximum count across all bins.
#[allow(dead_code)]
pub fn histogram_max_count(hist: &Histogram) -> u32 {
    hist.bins.iter().map(|b| b.count).max().unwrap_or(0)
}

/// Compute the weighted mean of the histogram.
#[allow(dead_code)]
pub fn histogram_mean(hist: &Histogram) -> f32 {
    if hist.total_samples == 0 {
        return 0.0;
    }
    let sum: f64 = hist
        .bins
        .iter()
        .map(|b| {
            let mid = ((b.low + b.high) / 2.0) as f64;
            mid * b.count as f64
        })
        .sum();
    (sum / hist.total_samples as f64) as f32
}

/// Return the index of the bin containing the median sample.
#[allow(dead_code)]
pub fn histogram_median_bin(hist: &Histogram) -> usize {
    if hist.total_samples == 0 {
        return 0;
    }
    let half = hist.total_samples.div_ceil(2);
    let mut cum = 0u32;
    for (i, b) in hist.bins.iter().enumerate() {
        cum += b.count;
        if cum >= half {
            return i;
        }
    }
    hist.bins.len().saturating_sub(1)
}

/// Return the approximate value at the given percentile (0..100).
#[allow(dead_code)]
pub fn histogram_percentile(hist: &Histogram, pct: f32) -> f32 {
    if hist.total_samples == 0 || hist.bins.is_empty() {
        return 0.0;
    }
    let target = (pct / 100.0 * hist.total_samples as f32).ceil() as u32;
    let target = target.max(1);
    let mut cum = 0u32;
    for b in &hist.bins {
        cum += b.count;
        if cum >= target {
            return (b.low + b.high) / 2.0;
        }
    }
    let last = &hist.bins[hist.bins.len() - 1];
    (last.low + last.high) / 2.0
}

/// Normalize histogram bin counts to the `[0, 1]` range relative to the maximum.
#[allow(dead_code)]
pub fn normalize_histogram(hist: &Histogram) -> Vec<f32> {
    let max_c = histogram_max_count(hist);
    if max_c == 0 {
        return vec![0.0; hist.bins.len()];
    }
    hist.bins
        .iter()
        .map(|b| b.count as f32 / max_c as f32)
        .collect()
}

/// Render a text-based ASCII bar chart of the histogram.
#[allow(dead_code)]
pub fn histogram_to_ascii(hist: &Histogram, bar_width: usize) -> String {
    let max_c = histogram_max_count(hist);
    let mut out = String::new();
    out.push_str(&format!("  {}\n", hist.title));
    for b in &hist.bins {
        let bar_len = if max_c == 0 {
            0
        } else {
            ((b.count as f64 / max_c as f64) * bar_width as f64) as usize
        };
        let bar: String = "#".repeat(bar_len);
        out.push_str(&format!(
            "  [{:>8.4}, {:>8.4}) | {:<width$} {}\n",
            b.low,
            b.high,
            bar,
            b.count,
            width = bar_width
        ));
    }
    out
}

/// Serialize a histogram to JSON.
#[allow(dead_code)]
pub fn histogram_to_json(hist: &Histogram) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"title\": \"{}\",\n", hist.title));
    out.push_str(&format!("  \"totalSamples\": {},\n", hist.total_samples));
    out.push_str("  \"bins\": [\n");
    for (i, b) in hist.bins.iter().enumerate() {
        let comma = if i + 1 < hist.bins.len() { "," } else { "" };
        out.push_str(&format!(
            "    {{\"low\": {:.6}, \"high\": {:.6}, \"count\": {}}}{comma}\n",
            b.low, b.high, b.count
        ));
    }
    out.push_str("  ]\n}");
    out
}

/// Build a cumulative histogram from an existing histogram.
#[allow(dead_code)]
pub fn cumulative_histogram(hist: &Histogram) -> Histogram {
    let mut cum_bins = hist.bins.clone();
    let mut running = 0u32;
    for b in &mut cum_bins {
        running += b.count;
        b.count = running;
    }
    Histogram {
        bins: cum_bins,
        total_samples: hist.total_samples,
        title: format!("{} (cumulative)", hist.title),
    }
}

/// Compute the Shannon entropy (in nats) of the histogram distribution.
#[allow(dead_code)]
pub fn histogram_entropy(hist: &Histogram) -> EntropyResult {
    if hist.total_samples == 0 {
        return 0.0;
    }
    let n = hist.total_samples as f64;
    let mut entropy = 0.0_f64;
    for b in &hist.bins {
        if b.count > 0 {
            let p = b.count as f64 / n;
            entropy -= p * p.ln();
        }
    }
    entropy
}

/// Merge two histograms that share the same bin layout.
/// Bin boundaries are taken from `a`. Counts are summed.
#[allow(dead_code)]
pub fn merge_histograms(a: &Histogram, b: &Histogram) -> Histogram {
    let len = a.bins.len().min(b.bins.len());
    let bins: Vec<HistogramBin> = (0..len)
        .map(|i| HistogramBin {
            low: a.bins[i].low,
            high: a.bins[i].high,
            count: a.bins[i].count + b.bins[i].count,
        })
        .collect();
    let total = a.total_samples + b.total_samples;
    Histogram {
        bins,
        total_samples: total,
        title: format!("{} + {}", a.title, b.title),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<f32> {
        vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.5]
    }

    fn build_sample_hist() -> Histogram {
        let cfg = HistogramConfig {
            bin_count: 10,
            range_min: 0.0,
            range_max: 1.0,
            title: "Test".to_string(),
        };
        build_histogram(&sample_data(), &cfg)
    }

    #[test]
    fn default_config_values() {
        let cfg = default_histogram_config();
        assert_eq!(cfg.bin_count, 32);
        assert!((cfg.range_min).abs() < 1e-6);
        assert!((cfg.range_max - 1.0).abs() < 1e-6);
    }

    #[test]
    fn build_histogram_total_samples() {
        let h = build_sample_hist();
        assert_eq!(h.total_samples, 10);
    }

    #[test]
    fn build_histogram_bin_count() {
        let h = build_sample_hist();
        assert_eq!(histogram_bin_count(&h), 10);
    }

    #[test]
    fn max_count_correct() {
        let h = build_sample_hist();
        // Bin [0.4, 0.5) should have 1 and [0.5, 0.6) should have 2
        assert!(histogram_max_count(&h) >= 1);
    }

    #[test]
    fn mean_within_range() {
        let h = build_sample_hist();
        let m = histogram_mean(&h);
        assert!(m > 0.0 && m < 1.0);
    }

    #[test]
    fn mean_empty_histogram() {
        let cfg = default_histogram_config();
        let h = build_histogram(&[], &cfg);
        assert!(histogram_mean(&h).abs() < 1e-6);
    }

    #[test]
    fn median_bin_within_range() {
        let h = build_sample_hist();
        let med = histogram_median_bin(&h);
        assert!(med < histogram_bin_count(&h));
    }

    #[test]
    fn percentile_50_is_median_area() {
        let h = build_sample_hist();
        let p50 = histogram_percentile(&h, 50.0);
        assert!(p50 > 0.0 && p50 < 1.0);
    }

    #[test]
    fn percentile_0_and_100() {
        let h = build_sample_hist();
        let p0 = histogram_percentile(&h, 0.0);
        let p100 = histogram_percentile(&h, 100.0);
        assert!(p0 <= p100);
    }

    #[test]
    fn normalize_histogram_max_is_one() {
        let h = build_sample_hist();
        let norm = normalize_histogram(&h);
        let max_val = norm.iter().cloned().fold(0.0_f32, f32::max);
        assert!((max_val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ascii_chart_contains_title() {
        let h = build_sample_hist();
        let text = histogram_to_ascii(&h, 20);
        assert!(text.contains("Test"));
    }

    #[test]
    fn json_contains_bins() {
        let h = build_sample_hist();
        let json = histogram_to_json(&h);
        assert!(json.contains("\"bins\""));
        assert!(json.contains("\"totalSamples\": 10"));
    }

    #[test]
    fn cumulative_last_bin_equals_total() {
        let h = build_sample_hist();
        let cum = cumulative_histogram(&h);
        let last_count = cum.bins.last().unwrap().count;
        assert_eq!(last_count, h.total_samples);
    }

    #[test]
    fn entropy_non_negative() {
        let h = build_sample_hist();
        assert!(histogram_entropy(&h) >= 0.0);
    }

    #[test]
    fn entropy_empty() {
        let cfg = default_histogram_config();
        let h = build_histogram(&[], &cfg);
        assert!(histogram_entropy(&h).abs() < 1e-12);
    }

    #[test]
    fn merge_histograms_sums_counts() {
        let h1 = build_sample_hist();
        let h2 = build_sample_hist();
        let merged = merge_histograms(&h1, &h2);
        assert_eq!(merged.total_samples, h1.total_samples + h2.total_samples);
        for (i, b) in merged.bins.iter().enumerate() {
            assert_eq!(b.count, h1.bins[i].count + h2.bins[i].count);
        }
    }

    #[test]
    fn out_of_range_values_excluded() {
        let cfg = HistogramConfig {
            bin_count: 5,
            range_min: 0.0,
            range_max: 1.0,
            title: "T".to_string(),
        };
        let data = vec![-1.0, 0.5, 2.0];
        let h = build_histogram(&data, &cfg);
        assert_eq!(h.total_samples, 1);
    }
}
