// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GPU perf — lightweight GPU performance counter tracking.

/// A single GPU performance counter entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuPerfEntry {
    pub name: String,
    pub duration_ns: u64,
    pub frame: u64,
}

/// GPU performance tracker.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GpuPerf {
    pub entries: Vec<GpuPerfEntry>,
    pub current_frame: u64,
}

#[allow(dead_code)]
pub fn new_gpu_perf() -> GpuPerf {
    GpuPerf::default()
}

#[allow(dead_code)]
pub fn gperf_record(perf: &mut GpuPerf, name: &str, duration_ns: u64) {
    perf.entries.push(GpuPerfEntry {
        name: name.to_string(),
        duration_ns,
        frame: perf.current_frame,
    });
}

#[allow(dead_code)]
pub fn gperf_begin_frame(perf: &mut GpuPerf) {
    perf.current_frame += 1;
}

#[allow(dead_code)]
pub fn gperf_clear(perf: &mut GpuPerf) {
    perf.entries.clear();
}

#[allow(dead_code)]
pub fn gperf_entry_count(perf: &GpuPerf) -> usize {
    perf.entries.len()
}

#[allow(dead_code)]
pub fn gperf_total_ns(perf: &GpuPerf) -> u64 {
    perf.entries.iter().map(|e| e.duration_ns).sum()
}

#[allow(dead_code)]
pub fn gperf_average_ns(perf: &GpuPerf) -> f64 {
    if perf.entries.is_empty() {
        return 0.0;
    }
    gperf_total_ns(perf) as f64 / perf.entries.len() as f64
}

#[allow(dead_code)]
pub fn gperf_max_ns(perf: &GpuPerf) -> u64 {
    perf.entries
        .iter()
        .map(|e| e.duration_ns)
        .max()
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn gperf_slowest_pass(perf: &GpuPerf) -> Option<&str> {
    perf.entries
        .iter()
        .max_by_key(|e| e.duration_ns)
        .map(|e| e.name.as_str())
}

#[allow(dead_code)]
pub fn gperf_frame_total_ns(perf: &GpuPerf, frame: u64) -> u64 {
    perf.entries
        .iter()
        .filter(|e| e.frame == frame)
        .map(|e| e.duration_ns)
        .sum()
}

#[allow(dead_code)]
pub fn gperf_to_json(perf: &GpuPerf) -> String {
    format!(
        r#"{{"frame":{},"count":{},"total_ns":{},"avg_ns":{:.1}}}"#,
        perf.current_frame,
        gperf_entry_count(perf),
        gperf_total_ns(perf),
        gperf_average_ns(perf)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_perf_empty() {
        let p = new_gpu_perf();
        assert_eq!(gperf_entry_count(&p), 0);
    }

    #[test]
    fn record_entry() {
        let mut p = new_gpu_perf();
        gperf_record(&mut p, "shadow", 1_000_000);
        assert_eq!(gperf_entry_count(&p), 1);
    }

    #[test]
    fn total_ns() {
        let mut p = new_gpu_perf();
        gperf_record(&mut p, "a", 1_000);
        gperf_record(&mut p, "b", 2_000);
        assert_eq!(gperf_total_ns(&p), 3_000);
    }

    #[test]
    fn average_ns() {
        let mut p = new_gpu_perf();
        gperf_record(&mut p, "a", 1_000);
        gperf_record(&mut p, "b", 3_000);
        assert!((gperf_average_ns(&p) - 2_000.0).abs() < 1.0);
    }

    #[test]
    fn max_ns() {
        let mut p = new_gpu_perf();
        gperf_record(&mut p, "a", 5_000);
        gperf_record(&mut p, "b", 1_000);
        assert_eq!(gperf_max_ns(&p), 5_000);
    }

    #[test]
    fn slowest_pass_name() {
        let mut p = new_gpu_perf();
        gperf_record(&mut p, "fast", 100);
        gperf_record(&mut p, "slow", 9_000);
        assert_eq!(gperf_slowest_pass(&p), Some("slow"));
    }

    #[test]
    fn begin_frame_increments() {
        let mut p = new_gpu_perf();
        gperf_begin_frame(&mut p);
        assert_eq!(p.current_frame, 1);
    }

    #[test]
    fn clear_removes_entries() {
        let mut p = new_gpu_perf();
        gperf_record(&mut p, "x", 1);
        gperf_clear(&mut p);
        assert_eq!(gperf_entry_count(&p), 0);
    }

    #[test]
    fn to_json_fields() {
        let p = new_gpu_perf();
        let j = gperf_to_json(&p);
        assert!(j.contains("frame"));
        assert!(j.contains("total_ns"));
    }
}
