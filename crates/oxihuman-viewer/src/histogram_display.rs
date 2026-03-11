//! Histogram display for debug — bins float values and generates ASCII/metadata representation.
//!
//! Accumulates float samples into fixed-width bins and exposes ASCII bar chart rendering,
//! normalization, and basic statistical queries.

/// Configuration for histogram construction and display.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistogramConfig {
    /// Default number of bins when using the config to create a histogram.
    pub default_bins: usize,
    /// Character used to draw each bar row.
    pub bar_char: char,
    /// Whether to show bin edges in ASCII output.
    pub show_edges: bool,
}

/// A single histogram bin storing its count and edge values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistogramBin {
    /// Left (inclusive) edge of the bin.
    pub low: f32,
    /// Right (exclusive) edge of the bin.
    pub high: f32,
    /// Number of samples that fell into this bin.
    pub count: u64,
}

/// A histogram data structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Ordered bins covering `[min, max)`.
    pub bins: Vec<HistogramBin>,
    /// Overall minimum of the histogram range.
    pub min: f32,
    /// Overall maximum of the histogram range.
    pub max: f32,
    /// Total number of samples added (including out-of-range, which are clamped).
    pub total_samples: u64,
}

/// Returns a sensible default [`HistogramConfig`].
#[allow(dead_code)]
pub fn default_histogram_config() -> HistogramConfig {
    HistogramConfig {
        default_bins: 10,
        bar_char: '#',
        show_edges: false,
    }
}

/// Creates a new [`Histogram`] covering `[min, max)` with `n_bins` equal-width bins.
///
/// # Panics
/// Panics if `n_bins` is zero or `min >= max`.
#[allow(dead_code)]
pub fn new_histogram(min: f32, max: f32, n_bins: usize) -> Histogram {
    assert!(n_bins > 0, "n_bins must be > 0");
    assert!(min < max, "min must be < max");
    let width = (max - min) / n_bins as f32;
    let bins = (0..n_bins)
        .map(|i| HistogramBin {
            low: min + i as f32 * width,
            high: min + (i + 1) as f32 * width,
            count: 0,
        })
        .collect();
    Histogram {
        bins,
        min,
        max,
        total_samples: 0,
    }
}

/// Adds a single float value to the histogram.
///
/// Values outside `[min, max)` are clamped to the nearest boundary bin.
#[allow(dead_code)]
pub fn histogram_add_value(hist: &mut Histogram, value: f32) {
    hist.total_samples += 1;
    if hist.bins.is_empty() {
        return;
    }
    let n = hist.bins.len();
    let t = (value - hist.min) / (hist.max - hist.min);
    let idx = (t * n as f32).floor() as isize;
    let idx = idx.clamp(0, (n as isize) - 1) as usize;
    hist.bins[idx].count += 1;
}

/// Returns the number of bins in the histogram.
#[allow(dead_code)]
pub fn histogram_bin_count(hist: &Histogram) -> usize {
    hist.bins.len()
}

/// Returns the total number of samples added to the histogram.
#[allow(dead_code)]
pub fn histogram_total_samples(hist: &Histogram) -> u64 {
    hist.total_samples
}

/// Returns the index of the bin with the highest count.
///
/// Returns `0` if the histogram is empty.
#[allow(dead_code)]
pub fn histogram_peak_bin(hist: &Histogram) -> usize {
    hist.bins
        .iter()
        .enumerate()
        .max_by_key(|(_, b)| b.count)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Returns the estimated mean of all added samples (mid-point weighted by bin count).
///
/// Returns `0.0` if no samples have been added.
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

/// Renders the histogram as an ASCII bar chart string.
///
/// `width` is the maximum bar width in characters.
#[allow(dead_code)]
pub fn histogram_to_ascii(hist: &Histogram, width: usize) -> String {
    if hist.bins.is_empty() {
        return String::new();
    }
    let max_count = hist.bins.iter().map(|b| b.count).max().unwrap_or(1).max(1);
    let mut out = String::new();
    for (i, bin) in hist.bins.iter().enumerate() {
        let bar_len = if width == 0 {
            0
        } else {
            (bin.count as f64 / max_count as f64 * width as f64).round() as usize
        };
        let bar = "#".repeat(bar_len);
        out.push_str(&format!("[{i:>3}] |{bar:<width$}| {}\n", bin.count, width = width));
    }
    out
}

/// Returns a normalized view of bin counts as fractions of the total samples.
///
/// Returns an empty `Vec` if no samples have been added.
#[allow(dead_code)]
pub fn histogram_normalize(hist: &Histogram) -> Vec<f32> {
    if hist.total_samples == 0 {
        return Vec::new();
    }
    hist.bins
        .iter()
        .map(|b| b.count as f32 / hist.total_samples as f32)
        .collect()
}

/// Resets all bin counts and the total sample counter to zero.
#[allow(dead_code)]
pub fn histogram_clear(hist: &mut Histogram) {
    for bin in &mut hist.bins {
        bin.count = 0;
    }
    hist.total_samples = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_histogram_config();
        assert_eq!(cfg.default_bins, 10);
        assert_eq!(cfg.bar_char, '#');
    }

    #[test]
    fn test_new_histogram_bin_count() {
        let hist = new_histogram(0.0, 1.0, 5);
        assert_eq!(histogram_bin_count(&hist), 5);
    }

    #[test]
    fn test_add_value_increments_total() {
        let mut hist = new_histogram(0.0, 10.0, 10);
        histogram_add_value(&mut hist, 5.0);
        histogram_add_value(&mut hist, 3.0);
        assert_eq!(histogram_total_samples(&hist), 2);
    }

    #[test]
    fn test_add_value_correct_bin() {
        let mut hist = new_histogram(0.0, 10.0, 10);
        histogram_add_value(&mut hist, 1.5); // should land in bin 1 (1.0..2.0)
        assert_eq!(hist.bins[1].count, 1);
    }

    #[test]
    fn test_add_value_clamped_below() {
        let mut hist = new_histogram(0.0, 10.0, 10);
        histogram_add_value(&mut hist, -5.0);
        assert_eq!(hist.bins[0].count, 1);
    }

    #[test]
    fn test_add_value_clamped_above() {
        let mut hist = new_histogram(0.0, 10.0, 10);
        histogram_add_value(&mut hist, 100.0);
        assert_eq!(hist.bins[9].count, 1);
    }

    #[test]
    fn test_peak_bin() {
        let mut hist = new_histogram(0.0, 10.0, 10);
        for _ in 0..5 {
            histogram_add_value(&mut hist, 7.5); // bin 7
        }
        histogram_add_value(&mut hist, 1.0); // bin 1
        assert_eq!(histogram_peak_bin(&hist), 7);
    }

    #[test]
    fn test_mean_single_bin() {
        let mut hist = new_histogram(0.0, 10.0, 10);
        histogram_add_value(&mut hist, 5.5); // bin 5, mid = 5.5
        let mean = histogram_mean(&hist);
        // mid-point of bin 5 is 5.5
        assert!((mean - 5.5).abs() < 0.5);
    }

    #[test]
    fn test_mean_no_samples() {
        let hist = new_histogram(0.0, 1.0, 4);
        assert_eq!(histogram_mean(&hist), 0.0);
    }

    #[test]
    fn test_to_ascii_nonempty() {
        let mut hist = new_histogram(0.0, 5.0, 5);
        histogram_add_value(&mut hist, 2.5);
        let s = histogram_to_ascii(&hist, 20);
        assert!(!s.is_empty());
        assert!(s.contains('#'));
    }

    #[test]
    fn test_normalize_sums_to_one() {
        let mut hist = new_histogram(0.0, 10.0, 5);
        for v in [1.0f32, 3.0, 5.0, 7.0, 9.0] {
            histogram_add_value(&mut hist, v);
        }
        let norm = histogram_normalize(&hist);
        let sum: f32 = norm.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_clear_resets_counts() {
        let mut hist = new_histogram(0.0, 10.0, 10);
        histogram_add_value(&mut hist, 5.0);
        histogram_clear(&mut hist);
        assert_eq!(histogram_total_samples(&hist), 0);
        assert!(hist.bins.iter().all(|b| b.count == 0));
    }
}
