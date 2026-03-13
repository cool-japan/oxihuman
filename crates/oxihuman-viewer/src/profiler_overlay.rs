// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! ProfilerOverlay — on-screen timing overlay.

#![allow(dead_code)]

/// A single profiler timing sample.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProfilerSample {
    pub label: String,
    pub ms: f32,
}

/// Profiler overlay accumulating named timing samples.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ProfilerOverlay {
    pub samples: Vec<ProfilerSample>,
}

/// Create an empty `ProfilerOverlay`.
#[allow(dead_code)]
pub fn new_profiler_overlay() -> ProfilerOverlay {
    ProfilerOverlay::default()
}

/// Record a timing sample.
#[allow(dead_code)]
pub fn record_sample(overlay: &mut ProfilerOverlay, label: &str, ms: f32) {
    overlay.samples.push(ProfilerSample { label: label.to_owned(), ms });
}

/// Return the number of samples.
#[allow(dead_code)]
pub fn sample_count(overlay: &ProfilerOverlay) -> usize {
    overlay.samples.len()
}

/// Return a reference to the sample at `index`.
#[allow(dead_code)]
pub fn sample_at(overlay: &ProfilerOverlay, index: usize) -> Option<&ProfilerSample> {
    overlay.samples.get(index)
}

/// Return the mean sample time in ms.
#[allow(dead_code)]
pub fn overlay_avg_ms(overlay: &ProfilerOverlay) -> f32 {
    if overlay.samples.is_empty() {
        return 0.0;
    }
    overlay.samples.iter().map(|s| s.ms).sum::<f32>() / overlay.samples.len() as f32
}

/// Return the maximum sample time in ms.
#[allow(dead_code)]
pub fn overlay_max_ms(overlay: &ProfilerOverlay) -> f32 {
    overlay.samples.iter().map(|s| s.ms).fold(0.0_f32, f32::max)
}

/// Return the minimum sample time in ms.
#[allow(dead_code)]
pub fn overlay_min_ms(overlay: &ProfilerOverlay) -> f32 {
    if overlay.samples.is_empty() {
        return 0.0;
    }
    overlay.samples.iter().map(|s| s.ms).fold(f32::INFINITY, f32::min)
}

/// Clear all samples.
#[allow(dead_code)]
pub fn profiler_reset(overlay: &mut ProfilerOverlay) {
    overlay.samples.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_profiler_empty() {
        let o = new_profiler_overlay();
        assert_eq!(sample_count(&o), 0);
    }

    #[test]
    fn test_record_sample() {
        let mut o = new_profiler_overlay();
        record_sample(&mut o, "render", 16.7);
        assert_eq!(sample_count(&o), 1);
    }

    #[test]
    fn test_sample_at_some() {
        let mut o = new_profiler_overlay();
        record_sample(&mut o, "gpu", 5.0);
        let s = sample_at(&o, 0).expect("should succeed");
        assert_eq!(s.label, "gpu");
        assert!((s.ms - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_sample_at_none() {
        let o = new_profiler_overlay();
        assert!(sample_at(&o, 0).is_none());
    }

    #[test]
    fn test_overlay_avg_ms() {
        let mut o = new_profiler_overlay();
        record_sample(&mut o, "a", 10.0);
        record_sample(&mut o, "b", 20.0);
        assert!((overlay_avg_ms(&o) - 15.0).abs() < 1e-5);
    }

    #[test]
    fn test_overlay_max_ms() {
        let mut o = new_profiler_overlay();
        record_sample(&mut o, "x", 5.0);
        record_sample(&mut o, "y", 12.0);
        assert!((overlay_max_ms(&o) - 12.0).abs() < 1e-5);
    }

    #[test]
    fn test_overlay_min_ms() {
        let mut o = new_profiler_overlay();
        record_sample(&mut o, "x", 3.0);
        record_sample(&mut o, "y", 8.0);
        assert!((overlay_min_ms(&o) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_profiler_reset() {
        let mut o = new_profiler_overlay();
        record_sample(&mut o, "z", 1.0);
        profiler_reset(&mut o);
        assert_eq!(sample_count(&o), 0);
    }

    #[test]
    fn test_avg_empty() {
        let o = new_profiler_overlay();
        assert!((overlay_avg_ms(&o)).abs() < 1e-6);
    }
}
