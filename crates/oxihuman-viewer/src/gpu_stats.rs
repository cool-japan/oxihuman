// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GPU statistics collector — frame-level draw calls, triangles, bandwidth.

/// Per-frame GPU statistics snapshot.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GpuFrameStats {
    pub frame_index: u64,
    pub draw_calls: u32,
    pub triangle_count: u64,
    pub vertex_count: u64,
    pub gpu_time_ns: u64,
    pub texture_bandwidth_bytes: u64,
    pub vertex_bandwidth_bytes: u64,
}

/// Rolling statistics buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuStats {
    history: Vec<GpuFrameStats>,
    capacity: usize,
    frame_index: u64,
}

impl GpuStats {
    #[allow(dead_code)]
    pub fn new(capacity: usize) -> Self {
        Self {
            history: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
            frame_index: 0,
        }
    }
}

#[allow(dead_code)]
pub fn new_gpu_stats(capacity: usize) -> GpuStats {
    GpuStats::new(capacity)
}

#[allow(dead_code)]
pub fn gs_push_frame(stats: &mut GpuStats, mut frame: GpuFrameStats) {
    stats.frame_index += 1;
    frame.frame_index = stats.frame_index;
    if stats.history.len() >= stats.capacity {
        stats.history.remove(0);
    }
    stats.history.push(frame);
}

#[allow(dead_code)]
pub fn gs_frame_count(stats: &GpuStats) -> usize {
    stats.history.len()
}

#[allow(dead_code)]
pub fn gs_last_frame(stats: &GpuStats) -> Option<&GpuFrameStats> {
    stats.history.last()
}

#[allow(dead_code)]
pub fn gs_average_gpu_time_ns(stats: &GpuStats) -> f64 {
    if stats.history.is_empty() {
        return 0.0;
    }
    let sum: u64 = stats.history.iter().map(|f| f.gpu_time_ns).sum();
    sum as f64 / stats.history.len() as f64
}

#[allow(dead_code)]
pub fn gs_average_draw_calls(stats: &GpuStats) -> f64 {
    if stats.history.is_empty() {
        return 0.0;
    }
    let sum: u64 = stats.history.iter().map(|f| f.draw_calls as u64).sum();
    sum as f64 / stats.history.len() as f64
}

#[allow(dead_code)]
pub fn gs_peak_triangle_count(stats: &GpuStats) -> u64 {
    stats
        .history
        .iter()
        .map(|f| f.triangle_count)
        .max()
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn gs_total_bandwidth_bytes(stats: &GpuStats) -> u64 {
    stats
        .history
        .iter()
        .map(|f| f.texture_bandwidth_bytes + f.vertex_bandwidth_bytes)
        .sum()
}

#[allow(dead_code)]
pub fn gs_clear(stats: &mut GpuStats) {
    stats.history.clear();
    stats.frame_index = 0;
}

#[allow(dead_code)]
pub fn gs_fps_from_ns(gpu_time_ns: u64) -> f32 {
    if gpu_time_ns == 0 {
        return 0.0;
    }
    1_000_000_000.0 / gpu_time_ns as f32
}

#[allow(dead_code)]
pub fn gs_to_json(stats: &GpuStats) -> String {
    let avg_ns = gs_average_gpu_time_ns(stats);
    let avg_dc = gs_average_draw_calls(stats);
    format!(
        "{{\"frames\":{},\"avg_gpu_ns\":{:.0},\"avg_draw_calls\":{:.1}}}",
        stats.history.len(),
        avg_ns,
        avg_dc
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(dc: u32, tris: u64, ns: u64) -> GpuFrameStats {
        GpuFrameStats {
            draw_calls: dc,
            triangle_count: tris,
            gpu_time_ns: ns,
            ..Default::default()
        }
    }

    #[test]
    fn empty_stats() {
        let s = new_gpu_stats(10);
        assert_eq!(gs_frame_count(&s), 0);
    }

    #[test]
    fn push_increments_count() {
        let mut s = new_gpu_stats(10);
        gs_push_frame(&mut s, make_frame(100, 5000, 8_000_000));
        assert_eq!(gs_frame_count(&s), 1);
    }

    #[test]
    fn capacity_evicts_oldest() {
        let mut s = new_gpu_stats(2);
        gs_push_frame(&mut s, make_frame(1, 0, 0));
        gs_push_frame(&mut s, make_frame(2, 0, 0));
        gs_push_frame(&mut s, make_frame(3, 0, 0));
        assert_eq!(gs_frame_count(&s), 2);
        assert_eq!(gs_last_frame(&s).expect("should succeed").draw_calls, 3);
    }

    #[test]
    fn average_gpu_time_correct() {
        let mut s = new_gpu_stats(10);
        gs_push_frame(&mut s, make_frame(0, 0, 10));
        gs_push_frame(&mut s, make_frame(0, 0, 20));
        assert!((gs_average_gpu_time_ns(&s) - 15.0).abs() < 1e-3);
    }

    #[test]
    fn peak_triangles() {
        let mut s = new_gpu_stats(10);
        gs_push_frame(&mut s, make_frame(0, 100, 0));
        gs_push_frame(&mut s, make_frame(0, 500, 0));
        assert_eq!(gs_peak_triangle_count(&s), 500);
    }

    #[test]
    fn clear_resets() {
        let mut s = new_gpu_stats(10);
        gs_push_frame(&mut s, make_frame(1, 1, 1));
        gs_clear(&mut s);
        assert_eq!(gs_frame_count(&s), 0);
    }

    #[test]
    fn fps_from_ns_reasonable() {
        let fps = gs_fps_from_ns(16_666_667);
        assert!((fps - 60.0).abs() < 1.0);
    }

    #[test]
    fn fps_zero_for_zero_ns() {
        assert!(gs_fps_from_ns(0) < 1e-6);
    }

    #[test]
    fn json_has_frames() {
        let s = new_gpu_stats(5);
        assert!(gs_to_json(&s).contains("frames"));
    }

    #[test]
    fn frame_index_increments() {
        let mut s = new_gpu_stats(5);
        gs_push_frame(&mut s, make_frame(0, 0, 0));
        gs_push_frame(&mut s, make_frame(0, 0, 0));
        assert_eq!(gs_last_frame(&s).expect("should succeed").frame_index, 2);
    }
}
