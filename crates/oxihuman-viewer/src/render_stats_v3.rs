// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Extended render statistics with ring-buffer history, fps, and p99 frame time.
//!
//! [`RenderStatsV3`] is the primary struct.  It embeds a [`FrameTimer`] that
//! keeps a ring buffer of the last 60 frame times for FPS and percentile
//! computation.

use crate::lod_manager_v2::LodLevelV2;

// ── FrameTimer ────────────────────────────────────────────────────────────────

/// Ring-buffer frame timer tracking the last `N` frame durations.
///
/// The ring buffer has a fixed capacity of [`FrameTimer::CAPACITY`] frames
/// (60 by default) to keep memory overhead constant.
#[derive(Debug, Clone)]
pub struct FrameTimer {
    /// Ring buffer of frame durations in milliseconds.
    buffer: Vec<f32>,
    /// Write head (index of the slot to overwrite next).
    head: usize,
    /// Number of valid samples currently stored.
    count: usize,
}

impl FrameTimer {
    /// Number of frame slots in the ring buffer.
    pub const CAPACITY: usize = 60;

    /// Create a new empty [`FrameTimer`].
    pub fn new() -> Self {
        FrameTimer {
            buffer: vec![0.0; Self::CAPACITY],
            head: 0,
            count: 0,
        }
    }

    /// Push a new frame duration (in milliseconds).
    pub fn push(&mut self, frame_ms: f32) {
        self.buffer[self.head] = frame_ms;
        self.head = (self.head + 1) % Self::CAPACITY;
        if self.count < Self::CAPACITY {
            self.count += 1;
        }
    }

    /// Number of valid samples in the buffer.
    pub fn sample_count(&self) -> usize {
        self.count
    }

    /// Return a slice of valid (possibly non-contiguous) frame times.
    ///
    /// The returned `Vec` is ordered from oldest to newest.
    pub fn samples(&self) -> Vec<f32> {
        if self.count == 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(self.count);
        // The oldest sample is at `head` when the buffer is full; otherwise
        // at index 0.
        let start = if self.count == Self::CAPACITY {
            self.head
        } else {
            0
        };
        for i in 0..self.count {
            out.push(self.buffer[(start + i) % Self::CAPACITY]);
        }
        out
    }

    /// Average frame time in milliseconds over all valid samples.
    ///
    /// Returns `0.0` if no samples are present.
    pub fn average_ms(&self) -> f32 {
        let s = self.samples();
        if s.is_empty() {
            return 0.0;
        }
        s.iter().sum::<f32>() / s.len() as f32
    }

    /// Frames per second derived from the average frame time.
    ///
    /// Returns `0.0` if the average frame time is zero.
    pub fn fps(&self) -> f32 {
        let avg = self.average_ms();
        if avg < f32::EPSILON {
            0.0
        } else {
            1000.0 / avg
        }
    }

    /// 99th-percentile frame time in milliseconds.
    ///
    /// Uses sorting; returns `0.0` if fewer than 2 samples exist.
    pub fn p99_frame_time_ms(&self) -> f32 {
        let mut s = self.samples();
        if s.len() < 2 {
            return s.first().copied().unwrap_or(0.0);
        }
        s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // Index for the 99th percentile using nearest-rank method.
        let idx = ((0.99 * s.len() as f32).ceil() as usize).saturating_sub(1);
        s[idx.min(s.len() - 1)]
    }

    /// Minimum frame time in the buffer (best frame).
    pub fn min_ms(&self) -> f32 {
        self.samples()
            .into_iter()
            .fold(f32::MAX, f32::min)
            .min(f32::MAX)
    }

    /// Maximum frame time in the buffer (worst frame).
    pub fn max_ms(&self) -> f32 {
        self.samples()
            .into_iter()
            .fold(f32::MIN, f32::max)
            .max(f32::MIN)
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        FrameTimer::new()
    }
}

// ── RenderStatsV3 ─────────────────────────────────────────────────────────────

/// Extended per-frame render statistics including LOD and morph tracking.
#[derive(Debug, Clone, Default)]
pub struct RenderStatsSnapshot {
    /// FPS averaged over the timer history window.
    pub fps: f32,
    /// Instantaneous frame time in milliseconds.
    pub frame_time_ms: f32,
    /// Number of triangles submitted this frame.
    pub triangle_count: u64,
    /// Number of GPU draw calls issued this frame.
    pub draw_calls: u32,
    /// GPU memory usage estimate in megabytes.
    pub gpu_memory_mb: f32,
    /// Currently active LOD level.
    pub lod_level: LodLevelV2,
    /// Number of morph sliders that were dirty this frame.
    pub morph_dirty_count: u32,
}

/// Comprehensive render statistics accumulator.
///
/// Use [`RenderStatsV3::begin_frame`] / [`RenderStatsV3::end_frame`] to
/// bracket each frame; call [`RenderStatsV3::snapshot`] to query the current
/// statistics.
#[derive(Debug)]
pub struct RenderStatsV3 {
    /// Ring-buffer timer for FPS and percentile computation.
    pub timer: FrameTimer,
    /// Total frames recorded since the last reset.
    pub frame_count: u64,
    /// In-progress frame start time (raw `std::time::Instant`).
    frame_start: Option<std::time::Instant>,
    /// Triangle count for the current in-progress frame.
    current_triangles: u64,
    /// Draw call count for the current in-progress frame.
    current_draw_calls: u32,
    /// Most recently finalised snapshot.
    last_snapshot: RenderStatsSnapshot,
    /// Estimated GPU memory usage in megabytes.
    pub gpu_memory_mb: f32,
    /// LOD level active at the last frame end.
    pub lod_level: LodLevelV2,
    /// Morph dirty count at the last frame end.
    pub morph_dirty_count: u32,
}

impl RenderStatsV3 {
    /// Create a new zeroed statistics tracker.
    pub fn new() -> Self {
        RenderStatsV3 {
            timer: FrameTimer::new(),
            frame_count: 0,
            frame_start: None,
            current_triangles: 0,
            current_draw_calls: 0,
            last_snapshot: RenderStatsSnapshot::default(),
            gpu_memory_mb: 0.0,
            lod_level: LodLevelV2::Full,
            morph_dirty_count: 0,
        }
    }

    /// Mark the start of a new frame.
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(std::time::Instant::now());
        self.current_triangles = 0;
        self.current_draw_calls = 0;
    }

    /// Record a draw call contributing `triangle_count` triangles.
    pub fn record_draw(&mut self, triangle_count: u32) {
        self.current_draw_calls += 1;
        self.current_triangles += u64::from(triangle_count);
    }

    /// Mark the end of the current frame, recording timing and finalising the snapshot.
    ///
    /// Call [`RenderStatsV3::begin_frame`] before this.  If `begin_frame` was
    /// not called, the frame time is recorded as `0.0 ms`.
    pub fn end_frame(&mut self) {
        let frame_ms = self
            .frame_start
            .take()
            .map(|t| t.elapsed().as_secs_f32() * 1000.0)
            .unwrap_or(0.0);

        self.timer.push(frame_ms);
        self.frame_count += 1;

        self.last_snapshot = RenderStatsSnapshot {
            fps: self.timer.fps(),
            frame_time_ms: frame_ms,
            triangle_count: self.current_triangles,
            draw_calls: self.current_draw_calls,
            gpu_memory_mb: self.gpu_memory_mb,
            lod_level: self.lod_level,
            morph_dirty_count: self.morph_dirty_count,
        };
    }

    /// Return the most recently finalised [`RenderStatsSnapshot`].
    pub fn snapshot(&self) -> &RenderStatsSnapshot {
        &self.last_snapshot
    }

    /// Frames per second averaged over the ring-buffer window.
    pub fn fps(&self) -> f32 {
        self.timer.fps()
    }

    /// 99th-percentile frame time in milliseconds over the ring-buffer window.
    pub fn p99_frame_time_ms(&self) -> f32 {
        self.timer.p99_frame_time_ms()
    }

    /// Reset all counters, clearing the ring buffer and frame history.
    pub fn reset(&mut self) {
        self.timer = FrameTimer::new();
        self.frame_count = 0;
        self.frame_start = None;
        self.current_triangles = 0;
        self.current_draw_calls = 0;
        self.last_snapshot = RenderStatsSnapshot::default();
    }

    /// Update the GPU memory estimate (call after uploading buffers).
    pub fn set_gpu_memory_mb(&mut self, mb: f32) {
        self.gpu_memory_mb = mb.max(0.0);
    }

    /// Update the active LOD level (call at the start of each frame).
    pub fn set_lod_level(&mut self, lod: LodLevelV2) {
        self.lod_level = lod;
    }

    /// Update the morph dirty slider count (call before `begin_frame`).
    pub fn set_morph_dirty_count(&mut self, count: u32) {
        self.morph_dirty_count = count;
    }

    /// Return a JSON summary string for logging or overlay display.
    pub fn to_json(&self) -> String {
        let snap = &self.last_snapshot;
        format!(
            r#"{{"fps":{:.1},"frame_time_ms":{:.2},"p99_ms":{:.2},"triangles":{},"draw_calls":{},"gpu_mb":{:.1},"lod":"{}","morph_dirty":{}}}"#,
            snap.fps,
            snap.frame_time_ms,
            self.p99_frame_time_ms(),
            snap.triangle_count,
            snap.draw_calls,
            snap.gpu_memory_mb,
            snap.lod_level.name(),
            snap.morph_dirty_count,
        )
    }
}

impl Default for RenderStatsV3 {
    fn default() -> Self {
        RenderStatsV3::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_timer_with_frames(frames: &[f32]) -> FrameTimer {
        let mut t = FrameTimer::new();
        for &f in frames {
            t.push(f);
        }
        t
    }

    #[test]
    fn frame_timer_push_increments_count() {
        let mut t = FrameTimer::new();
        t.push(16.0);
        assert_eq!(t.sample_count(), 1);
    }

    #[test]
    fn frame_timer_capacity_is_60() {
        assert_eq!(FrameTimer::CAPACITY, 60);
    }

    #[test]
    fn frame_timer_ring_buffer_wraps_at_capacity() {
        let mut t = FrameTimer::new();
        for i in 0..70u32 {
            t.push(i as f32);
        }
        assert_eq!(t.sample_count(), 60, "count should not exceed capacity");
    }

    #[test]
    fn frame_timer_average_ms_correct() {
        let t = make_timer_with_frames(&[10.0, 20.0, 30.0]);
        let avg = t.average_ms();
        assert!((avg - 20.0).abs() < 1e-4, "avg should be 20, got {avg}");
    }

    #[test]
    fn frame_timer_fps_at_16ms() {
        let t = make_timer_with_frames(&[16.666_67f32; 60]);
        let fps = t.fps();
        assert!((fps - 60.0).abs() < 0.5, "fps should be ~60, got {fps}");
    }

    #[test]
    fn frame_timer_fps_zero_when_empty() {
        let t = FrameTimer::new();
        assert_eq!(t.fps(), 0.0);
    }

    #[test]
    fn frame_timer_p99_single_sample() {
        let t = make_timer_with_frames(&[20.0]);
        assert!((t.p99_frame_time_ms() - 20.0).abs() < 1e-4);
    }

    #[test]
    fn frame_timer_p99_sorted_correctness() {
        // 60 frames: 59 at 16 ms and 1 spike at 100 ms.
        let mut frames = vec![16.0f32; 59];
        frames.push(100.0);
        let t = make_timer_with_frames(&frames);
        let p99 = t.p99_frame_time_ms();
        // The 99th percentile index for 60 samples = ceil(0.99 * 60) - 1 = 58
        // Sorted: [16, 16, ...(58 entries), 100]  → index 58 = 16.0
        assert!(
            (16.0..=100.0).contains(&p99),
            "p99 should be <=100 and >=16, got {p99}"
        );
    }

    #[test]
    fn frame_timer_samples_ordered_oldest_first() {
        let mut t = FrameTimer::new();
        for i in 0..65u32 {
            // overfill so we wrap
            t.push(i as f32);
        }
        let s = t.samples();
        assert_eq!(s.len(), 60);
        // The oldest should be 5 (65 - 60) and newest 64.
        assert!((s[0] - 5.0).abs() < f32::EPSILON, "oldest = {}", s[0]);
        assert!((s[59] - 64.0).abs() < f32::EPSILON, "newest = {}", s[59]);
    }

    #[test]
    fn render_stats_v3_frame_count_increments() {
        let mut stats = RenderStatsV3::new();
        stats.begin_frame();
        stats.end_frame();
        stats.begin_frame();
        stats.end_frame();
        assert_eq!(stats.frame_count, 2);
    }

    #[test]
    fn render_stats_v3_draw_call_accumulates() {
        let mut stats = RenderStatsV3::new();
        stats.begin_frame();
        stats.record_draw(1000);
        stats.record_draw(500);
        stats.end_frame();
        let snap = stats.snapshot();
        assert_eq!(snap.draw_calls, 2);
        assert_eq!(snap.triangle_count, 1500);
    }

    #[test]
    fn render_stats_v3_reset_clears_history() {
        let mut stats = RenderStatsV3::new();
        stats.begin_frame();
        stats.record_draw(999);
        stats.end_frame();
        stats.reset();
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.timer.sample_count(), 0);
    }

    #[test]
    fn render_stats_v3_lod_level_stored() {
        let mut stats = RenderStatsV3::new();
        stats.set_lod_level(LodLevelV2::Medium);
        stats.begin_frame();
        stats.end_frame();
        assert_eq!(stats.snapshot().lod_level, LodLevelV2::Medium);
    }

    #[test]
    fn render_stats_v3_morph_dirty_count_stored() {
        let mut stats = RenderStatsV3::new();
        stats.set_morph_dirty_count(7);
        stats.begin_frame();
        stats.end_frame();
        assert_eq!(stats.snapshot().morph_dirty_count, 7);
    }

    #[test]
    fn render_stats_v3_gpu_memory_non_negative() {
        let mut stats = RenderStatsV3::new();
        stats.set_gpu_memory_mb(-5.0);
        assert!(stats.gpu_memory_mb >= 0.0);
    }

    #[test]
    fn render_stats_v3_to_json_contains_fps() {
        let mut stats = RenderStatsV3::new();
        stats.begin_frame();
        stats.end_frame();
        let json = stats.to_json();
        assert!(json.contains("fps"), "json should contain fps: {json}");
    }

    #[test]
    fn render_stats_v3_fps_reflects_timer() {
        let mut stats = RenderStatsV3::new();
        // Manually push frame times directly via the timer.
        for _ in 0..60 {
            stats.timer.push(16.666_67);
        }
        let fps = stats.fps();
        assert!((fps - 60.0).abs() < 1.0, "expected ~60 fps, got {fps}");
    }

    #[test]
    fn render_stats_v3_p99_delegates_to_timer() {
        let mut stats = RenderStatsV3::new();
        for _ in 0..60 {
            stats.timer.push(8.333_33); // 120 fps
        }
        let p99 = stats.p99_frame_time_ms();
        assert!(
            (p99 - 8.333_33).abs() < 0.1,
            "p99 should be ~8.3 ms, got {p99}"
        );
    }
}
