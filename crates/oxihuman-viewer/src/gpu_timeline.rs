// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GPU timeline — records and analyzes per-pass GPU timing data.

/// A GPU timing sample.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct GpuTimeSample {
    pub pass_name: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub frame: u64,
}

impl GpuTimeSample {
    #[allow(dead_code)]
    pub fn duration_ns(&self) -> u64 {
        self.end_ns.saturating_sub(self.start_ns)
    }
    #[allow(dead_code)]
    pub fn duration_ms(&self) -> f32 {
        self.duration_ns() as f32 / 1_000_000.0
    }
}

/// GPU timeline configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct GpuTimelineConfig {
    pub max_samples: usize,
    pub enabled: bool,
}

impl Default for GpuTimelineConfig {
    fn default() -> Self {
        Self {
            max_samples: 4096,
            enabled: true,
        }
    }
}

/// GPU timeline state.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct GpuTimeline {
    pub config: GpuTimelineConfig,
    pub samples: Vec<GpuTimeSample>,
    pub current_frame: u64,
}

/// Create new GPU timeline.
#[allow(dead_code)]
pub fn new_gpu_timeline(cfg: GpuTimelineConfig) -> GpuTimeline {
    GpuTimeline {
        config: cfg,
        samples: Vec::new(),
        current_frame: 0,
    }
}

/// Begin a new frame.
#[allow(dead_code)]
pub fn begin_frame(t: &mut GpuTimeline) {
    t.current_frame += 1;
}

/// Record a timing sample.
#[allow(dead_code)]
pub fn record_sample(t: &mut GpuTimeline, name: &str, start_ns: u64, end_ns: u64) -> bool {
    if t.samples.len() >= t.config.max_samples {
        return false;
    }
    t.samples.push(GpuTimeSample {
        pass_name: name.to_string(),
        start_ns,
        end_ns,
        frame: t.current_frame,
    });
    true
}

/// Sample count.
#[allow(dead_code)]
pub fn sample_count(t: &GpuTimeline) -> usize {
    t.samples.len()
}

/// Clear all samples.
#[allow(dead_code)]
pub fn clear_timeline(t: &mut GpuTimeline) {
    t.samples.clear();
}

/// Total GPU time for the current frame in nanoseconds.
#[allow(dead_code)]
pub fn frame_total_ns(t: &GpuTimeline) -> u64 {
    t.samples
        .iter()
        .filter(|s| s.frame == t.current_frame)
        .map(|s| s.duration_ns())
        .sum()
}

/// Find the slowest pass name.
#[allow(dead_code)]
pub fn slowest_pass(t: &GpuTimeline) -> Option<&str> {
    t.samples
        .iter()
        .max_by_key(|s| s.duration_ns())
        .map(|s| s.pass_name.as_str())
}

/// Average duration across all samples in milliseconds.
#[allow(dead_code)]
pub fn average_duration_ms(t: &GpuTimeline) -> f32 {
    if t.samples.is_empty() {
        return 0.0;
    }
    let total: u64 = t.samples.iter().map(|s| s.duration_ns()).sum();
    (total as f32 / t.samples.len() as f32) / 1_000_000.0
}

/// Check if any sample exceeds a threshold in milliseconds.
#[allow(dead_code)]
pub fn has_spike(t: &GpuTimeline, threshold_ms: f32) -> bool {
    t.samples.iter().any(|s| s.duration_ms() > threshold_ms)
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn gpu_timeline_to_json(t: &GpuTimeline) -> String {
    format!(
        r#"{{"sample_count":{},"frame":{},"avg_ms":{:.4}}}"#,
        t.samples.len(),
        t.current_frame,
        average_duration_ms(t)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_timeline_empty() {
        let t = new_gpu_timeline(GpuTimelineConfig::default());
        assert_eq!(sample_count(&t), 0);
    }

    #[test]
    fn record_sample_ok() {
        let mut t = new_gpu_timeline(GpuTimelineConfig::default());
        assert!(record_sample(&mut t, "shadow", 0, 1_000_000));
    }

    #[test]
    fn sample_duration_ns() {
        let s = GpuTimeSample {
            pass_name: "test".to_string(),
            start_ns: 100,
            end_ns: 200,
            frame: 0,
        };
        assert_eq!(s.duration_ns(), 100);
    }

    #[test]
    fn duration_ms() {
        let s = GpuTimeSample {
            pass_name: "p".to_string(),
            start_ns: 0,
            end_ns: 1_000_000,
            frame: 0,
        };
        assert!((s.duration_ms() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn frame_increments() {
        let mut t = new_gpu_timeline(GpuTimelineConfig::default());
        begin_frame(&mut t);
        assert_eq!(t.current_frame, 1);
    }

    #[test]
    fn slowest_pass_found() {
        let mut t = new_gpu_timeline(GpuTimelineConfig::default());
        record_sample(&mut t, "fast", 0, 100);
        record_sample(&mut t, "slow", 0, 1_000_000);
        assert_eq!(slowest_pass(&t), Some("slow"));
    }

    #[test]
    fn average_ms_correct() {
        let mut t = new_gpu_timeline(GpuTimelineConfig::default());
        record_sample(&mut t, "a", 0, 2_000_000);
        record_sample(&mut t, "b", 0, 2_000_000);
        assert!((average_duration_ms(&t) - 2.0).abs() < 1e-3);
    }

    #[test]
    fn has_spike_detects() {
        let mut t = new_gpu_timeline(GpuTimelineConfig::default());
        record_sample(&mut t, "heavy", 0, 20_000_000);
        assert!(has_spike(&t, 10.0));
    }

    #[test]
    fn clear_removes_all() {
        let mut t = new_gpu_timeline(GpuTimelineConfig::default());
        record_sample(&mut t, "x", 0, 1);
        clear_timeline(&mut t);
        assert_eq!(sample_count(&t), 0);
    }

    #[test]
    fn json_contains_sample_count() {
        let t = new_gpu_timeline(GpuTimelineConfig::default());
        assert!(gpu_timeline_to_json(&t).contains("sample_count"));
    }
}
