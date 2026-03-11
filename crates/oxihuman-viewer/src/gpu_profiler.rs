// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GPU pass profiler (stub — records timing markers and computes statistics).

/// A single profiler marker.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct GpuMarker {
    pub name: String,
    pub duration_ns: u64,
    pub frame: u64,
}

/// Profiler state.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct GpuProfiler {
    pub markers: Vec<GpuMarker>,
    pub current_frame: u64,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_gpu_profiler() -> GpuProfiler {
    GpuProfiler {
        enabled: true,
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn gp_begin_frame(p: &mut GpuProfiler) {
    p.current_frame += 1;
}

#[allow(dead_code)]
pub fn gp_record(p: &mut GpuProfiler, name: &str, duration_ns: u64) {
    if !p.enabled {
        return;
    }
    p.markers.push(GpuMarker {
        name: name.to_string(),
        duration_ns,
        frame: p.current_frame,
    });
}

#[allow(dead_code)]
pub fn gp_clear(p: &mut GpuProfiler) {
    p.markers.clear();
}

#[allow(dead_code)]
pub fn gp_marker_count(p: &GpuProfiler) -> usize {
    p.markers.len()
}

#[allow(dead_code)]
pub fn gp_total_ns(p: &GpuProfiler) -> u64 {
    p.markers.iter().map(|m| m.duration_ns).sum()
}

#[allow(dead_code)]
pub fn gp_average_ns(p: &GpuProfiler) -> f64 {
    if p.markers.is_empty() {
        return 0.0;
    }
    p.markers.iter().map(|m| m.duration_ns as f64).sum::<f64>() / p.markers.len() as f64
}

#[allow(dead_code)]
pub fn gp_max_ns(p: &GpuProfiler) -> u64 {
    p.markers.iter().map(|m| m.duration_ns).max().unwrap_or(0)
}

/// Find the slowest marker name.
#[allow(dead_code)]
pub fn gp_slowest_pass(p: &GpuProfiler) -> Option<&str> {
    p.markers
        .iter()
        .max_by_key(|m| m.duration_ns)
        .map(|m| m.name.as_str())
}

#[allow(dead_code)]
pub fn gp_markers_in_frame(p: &GpuProfiler, frame: u64) -> Vec<&GpuMarker> {
    p.markers.iter().filter(|m| m.frame == frame).collect()
}

#[allow(dead_code)]
pub fn gp_to_json(p: &GpuProfiler) -> String {
    format!(
        "{{\"frame\":{},\"markers\":{},\"total_ns\":{},\"enabled\":{}}}",
        p.current_frame,
        p.markers.len(),
        gp_total_ns(p),
        p.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_enabled() {
        assert!(new_gpu_profiler().enabled);
    }

    #[test]
    fn begin_frame_increments() {
        let mut p = new_gpu_profiler();
        gp_begin_frame(&mut p);
        assert_eq!(p.current_frame, 1);
    }

    #[test]
    fn record_marker() {
        let mut p = new_gpu_profiler();
        gp_record(&mut p, "shadows", 500_000);
        assert_eq!(gp_marker_count(&p), 1);
    }

    #[test]
    fn disabled_does_not_record() {
        let mut p = new_gpu_profiler();
        p.enabled = false;
        gp_record(&mut p, "shadows", 1_000);
        assert_eq!(gp_marker_count(&p), 0);
    }

    #[test]
    fn total_ns_sum() {
        let mut p = new_gpu_profiler();
        gp_record(&mut p, "a", 100);
        gp_record(&mut p, "b", 200);
        assert_eq!(gp_total_ns(&p), 300);
    }

    #[test]
    fn average_ns_correct() {
        let mut p = new_gpu_profiler();
        gp_record(&mut p, "a", 100);
        gp_record(&mut p, "b", 300);
        assert!((gp_average_ns(&p) - 200.0).abs() < 1e-3);
    }

    #[test]
    fn slowest_pass() {
        let mut p = new_gpu_profiler();
        gp_record(&mut p, "fast", 100);
        gp_record(&mut p, "slow", 9999);
        assert_eq!(gp_slowest_pass(&p), Some("slow"));
    }

    #[test]
    fn clear_empties() {
        let mut p = new_gpu_profiler();
        gp_record(&mut p, "x", 1);
        gp_clear(&mut p);
        assert_eq!(gp_marker_count(&p), 0);
    }

    #[test]
    fn markers_in_frame() {
        let mut p = new_gpu_profiler();
        gp_begin_frame(&mut p);
        gp_record(&mut p, "x", 1);
        assert_eq!(gp_markers_in_frame(&p, 1).len(), 1);
    }

    #[test]
    fn json_has_frame() {
        assert!(gp_to_json(&new_gpu_profiler()).contains("frame"));
    }
}
