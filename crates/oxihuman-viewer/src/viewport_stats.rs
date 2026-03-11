// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Viewport rendering statistics panel.
//!
//! Records per-frame metrics (draw calls, vertices, triangles, frame time) and
//! exposes rolling averages, peak values, and text/JSON serialisation.

#![allow(dead_code)]

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for the viewport statistics panel.
#[derive(Debug, Clone)]
pub struct ViewportStatsConfig {
    /// Number of frames kept in the rolling window. Default: `60`.
    pub window_size: usize,
    /// Whether the stats panel is visible. Default: `true`.
    pub visible: bool,
    /// Maximum number of draw calls before triggering a warning flag.
    pub draw_call_warn_threshold: u32,
}

impl Default for ViewportStatsConfig {
    fn default() -> Self {
        Self {
            window_size: 60,
            visible: true,
            draw_call_warn_threshold: 1000,
        }
    }
}

/// Statistics captured for a single rendered frame.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    /// Frame duration in milliseconds.
    pub frame_time_ms: f32,
    /// Number of GPU draw calls issued.
    pub draw_calls: u32,
    /// Total vertex count submitted.
    pub vertex_count: u64,
    /// Total triangle count submitted.
    pub triangle_count: u64,
}

/// Viewport statistics accumulator.
#[derive(Debug, Clone)]
pub struct ViewportStats {
    /// User configuration.
    pub config: ViewportStatsConfig,
    /// Ring buffer of recent frame stats (up to `config.window_size` entries).
    history: Vec<FrameStats>,
    /// Total frames ever recorded.
    total_frames: u64,
    /// Peak frame time seen since creation or last reset.
    peak_ms: f32,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Average statistics over a rolling window.
pub type RollingAvg = (f32, u32, u64, u64);

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`ViewportStatsConfig`].
#[allow(dead_code)]
pub fn default_viewport_stats_config() -> ViewportStatsConfig {
    ViewportStatsConfig::default()
}

/// Create a new [`ViewportStats`] with the given configuration.
#[allow(dead_code)]
pub fn new_viewport_stats(config: ViewportStatsConfig) -> ViewportStats {
    ViewportStats {
        config,
        history: Vec::new(),
        total_frames: 0,
        peak_ms: 0.0,
    }
}

/// Record a new frame into `stats` and return it.
#[allow(dead_code)]
pub fn record_frame(mut stats: ViewportStats, frame: FrameStats) -> ViewportStats {
    if frame.frame_time_ms > stats.peak_ms {
        stats.peak_ms = frame.frame_time_ms;
    }
    stats.total_frames += 1;
    stats.history.push(frame);
    // Trim to window
    let win = stats.config.window_size;
    if stats.history.len() > win {
        let excess = stats.history.len() - win;
        stats.history.drain(..excess);
    }
    stats
}

/// Return the most recent frame time in milliseconds, or `0.0` if no frames
/// have been recorded.
#[allow(dead_code)]
pub fn frame_time_ms(stats: &ViewportStats) -> f32 {
    stats.history.last().map(|f| f.frame_time_ms).unwrap_or(0.0)
}

/// Return the most recent draw call count, or `0` if no frames recorded.
#[allow(dead_code)]
pub fn draw_call_count(stats: &ViewportStats) -> u32 {
    stats.history.last().map(|f| f.draw_calls).unwrap_or(0)
}

/// Return the most recent vertex count, or `0` if no frames recorded.
#[allow(dead_code)]
pub fn vertex_count_stat(stats: &ViewportStats) -> u64 {
    stats.history.last().map(|f| f.vertex_count).unwrap_or(0)
}

/// Return the most recent triangle count, or `0` if no frames recorded.
#[allow(dead_code)]
pub fn triangle_count_stat(stats: &ViewportStats) -> u64 {
    stats.history.last().map(|f| f.triangle_count).unwrap_or(0)
}

/// Compute the rolling average over the current window.
///
/// Returns `(avg_frame_time_ms, avg_draw_calls, avg_vertex_count, avg_triangle_count)`.
/// Returns `(0.0, 0, 0, 0)` if no frames have been recorded.
#[allow(dead_code)]
pub fn stats_rolling_average(stats: &ViewportStats) -> RollingAvg {
    let n = stats.history.len();
    if n == 0 {
        return (0.0, 0, 0, 0);
    }
    let n_f = n as f32;
    let n_u = n as u64;
    let sum_ms: f32 = stats.history.iter().map(|f| f.frame_time_ms).sum();
    let sum_dc: u64 = stats.history.iter().map(|f| f.draw_calls as u64).sum();
    let sum_vc: u64 = stats.history.iter().map(|f| f.vertex_count).sum();
    let sum_tc: u64 = stats.history.iter().map(|f| f.triangle_count).sum();
    (sum_ms / n_f, (sum_dc / n_u) as u32, sum_vc / n_u, sum_tc / n_u)
}

/// Format the current statistics as a human-readable text string.
#[allow(dead_code)]
pub fn stats_to_text(stats: &ViewportStats) -> String {
    let (avg_ms, avg_dc, avg_vc, avg_tc) = stats_rolling_average(stats);
    format!(
        "Frame: {:.2}ms  DrawCalls: {}  Verts: {}  Tris: {}  Avg: {:.2}ms/{}/{}/{}  Peak: {:.2}ms  Frames: {}",
        frame_time_ms(stats),
        draw_call_count(stats),
        vertex_count_stat(stats),
        triangle_count_stat(stats),
        avg_ms,
        avg_dc,
        avg_vc,
        avg_tc,
        stats.peak_ms,
        stats.total_frames,
    )
}

/// Serialise the current statistics as a JSON string.
#[allow(dead_code)]
pub fn stats_to_json(stats: &ViewportStats) -> String {
    let (avg_ms, avg_dc, avg_vc, avg_tc) = stats_rolling_average(stats);
    format!(
        "{{\"frame_time_ms\":{:.4},\"draw_calls\":{},\"vertex_count\":{},\"triangle_count\":{},\
         \"avg_frame_time_ms\":{:.4},\"avg_draw_calls\":{},\"avg_vertex_count\":{},\
         \"avg_triangle_count\":{},\"peak_frame_time_ms\":{:.4},\"total_frames\":{}}}",
        frame_time_ms(stats),
        draw_call_count(stats),
        vertex_count_stat(stats),
        triangle_count_stat(stats),
        avg_ms,
        avg_dc,
        avg_vc,
        avg_tc,
        stats.peak_ms,
        stats.total_frames,
    )
}

/// Reset all recorded history and counters in `stats`, returning it.
#[allow(dead_code)]
pub fn reset_stats(mut stats: ViewportStats) -> ViewportStats {
    stats.history.clear();
    stats.total_frames = 0;
    stats.peak_ms = 0.0;
    stats
}

/// Return the peak frame time in milliseconds since creation or last reset.
#[allow(dead_code)]
pub fn peak_frame_time_ms(stats: &ViewportStats) -> f32 {
    stats.peak_ms
}

/// Return whether the stats panel is currently set to visible.
#[allow(dead_code)]
pub fn stats_visible(stats: &ViewportStats) -> bool {
    stats.config.visible
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(ms: f32, dc: u32, vc: u64, tc: u64) -> FrameStats {
        FrameStats {
            frame_time_ms: ms,
            draw_calls: dc,
            vertex_count: vc,
            triangle_count: tc,
        }
    }

    fn filled_stats() -> ViewportStats {
        let cfg = default_viewport_stats_config();
        let s = new_viewport_stats(cfg);
        let s = record_frame(s, make_frame(16.0, 100, 10000, 5000));
        let s = record_frame(s, make_frame(18.0, 120, 12000, 6000));
        record_frame(s, make_frame(14.0, 80, 8000, 4000))
    }

    #[test]
    fn default_config_fields() {
        let cfg = default_viewport_stats_config();
        assert_eq!(cfg.window_size, 60);
        assert!(cfg.visible);
        assert_eq!(cfg.draw_call_warn_threshold, 1000);
    }

    #[test]
    fn new_stats_has_no_history() {
        let s = new_viewport_stats(default_viewport_stats_config());
        assert_eq!(s.history.len(), 0);
        assert_eq!(s.total_frames, 0);
    }

    #[test]
    fn record_frame_increments_total() {
        let s = new_viewport_stats(default_viewport_stats_config());
        let s = record_frame(s, make_frame(16.0, 50, 1000, 500));
        assert_eq!(s.total_frames, 1);
    }

    #[test]
    fn record_frame_updates_history() {
        let s = filled_stats();
        assert_eq!(s.history.len(), 3);
    }

    #[test]
    fn frame_time_ms_returns_last() {
        let s = filled_stats();
        assert!((frame_time_ms(&s) - 14.0).abs() < 1e-4);
    }

    #[test]
    fn draw_call_count_returns_last() {
        let s = filled_stats();
        assert_eq!(draw_call_count(&s), 80);
    }

    #[test]
    fn vertex_count_stat_returns_last() {
        let s = filled_stats();
        assert_eq!(vertex_count_stat(&s), 8000);
    }

    #[test]
    fn triangle_count_stat_returns_last() {
        let s = filled_stats();
        assert_eq!(triangle_count_stat(&s), 4000);
    }

    #[test]
    fn rolling_average_frame_time() {
        let s = filled_stats();
        let (avg_ms, _, _, _) = stats_rolling_average(&s);
        let expected = (16.0 + 18.0 + 14.0) / 3.0;
        assert!((avg_ms - expected).abs() < 0.01);
    }

    #[test]
    fn rolling_average_empty_returns_zeros() {
        let s = new_viewport_stats(default_viewport_stats_config());
        let avg = stats_rolling_average(&s);
        assert_eq!(avg, (0.0, 0, 0, 0));
    }

    #[test]
    fn stats_to_text_contains_ms() {
        let s = filled_stats();
        let t = stats_to_text(&s);
        assert!(t.contains("ms"));
    }

    #[test]
    fn stats_to_json_is_valid_json_start() {
        let s = filled_stats();
        let j = stats_to_json(&s);
        assert!(j.starts_with('{'));
        assert!(j.ends_with('}'));
    }

    #[test]
    fn reset_clears_history() {
        let s = filled_stats();
        let s = reset_stats(s);
        assert_eq!(s.history.len(), 0);
        assert_eq!(s.total_frames, 0);
        assert!((s.peak_ms).abs() < 1e-6);
    }

    #[test]
    fn peak_frame_time_ms_tracks_max() {
        let s = filled_stats();
        assert!((peak_frame_time_ms(&s) - 18.0).abs() < 1e-4);
    }

    #[test]
    fn stats_visible_reflects_config() {
        let mut cfg = default_viewport_stats_config();
        cfg.visible = false;
        let s = new_viewport_stats(cfg);
        assert!(!stats_visible(&s));
    }

    #[test]
    fn window_size_limits_history() {
        let mut cfg = default_viewport_stats_config();
        cfg.window_size = 2;
        let s = new_viewport_stats(cfg);
        let s = record_frame(s, make_frame(10.0, 1, 100, 50));
        let s = record_frame(s, make_frame(11.0, 2, 200, 100));
        let s = record_frame(s, make_frame(12.0, 3, 300, 150));
        // Window is 2 — only last 2 frames kept
        assert_eq!(s.history.len(), 2);
        assert!((s.history[0].frame_time_ms - 11.0).abs() < 1e-4);
    }

    #[test]
    fn frame_stats_default_is_zero() {
        let f = FrameStats::default();
        assert_eq!(f.draw_calls, 0);
        assert_eq!(f.vertex_count, 0);
        assert_eq!(f.triangle_count, 0);
        assert!((f.frame_time_ms).abs() < 1e-6);
    }
}
