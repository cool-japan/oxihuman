//! Rendering statistics tracker.
//!
//! Tracks per-frame metrics such as frame time, draw call count, triangle count, and
//! GPU memory usage across frames.

/// Configuration for the render stats tracker.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderStatsConfig {
    /// Number of recent frames to retain in the history ring buffer.
    pub history_capacity: usize,
    /// Whether to track per-frame statistics history.
    pub enable_history: bool,
}

/// Statistics recorded for a single rendered frame.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    /// Timestamp (seconds) when the frame began.
    pub begin_time: f64,
    /// Timestamp (seconds) when the frame ended.
    pub end_time: f64,
    /// Number of draw calls issued during the frame.
    pub draw_calls: u32,
    /// Total number of triangles submitted during the frame.
    pub triangle_count: u64,
}

/// Render statistics tracker.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderStats {
    /// Tracker configuration.
    pub config: RenderStatsConfig,
    /// Statistics for the current (in-progress) frame.
    pub current_frame: FrameStats,
    /// Statistics finalized for the most recently completed frame.
    pub last_frame: FrameStats,
    /// Cumulative draw call count since the last reset.
    pub total_draw_calls: u32,
    /// Cumulative triangle count since the last reset.
    pub total_triangles: u64,
    /// Number of completed frames since the last reset.
    pub frame_count: u64,
}

/// Returns a sensible default [`RenderStatsConfig`].
#[allow(dead_code)]
pub fn default_render_stats_config() -> RenderStatsConfig {
    RenderStatsConfig {
        history_capacity: 60,
        enable_history: true,
    }
}

/// Creates a new [`RenderStats`] tracker using the given configuration.
#[allow(dead_code)]
pub fn new_render_stats(cfg: &RenderStatsConfig) -> RenderStats {
    RenderStats {
        config: cfg.clone(),
        current_frame: FrameStats::default(),
        last_frame: FrameStats::default(),
        total_draw_calls: 0,
        total_triangles: 0,
        frame_count: 0,
    }
}

/// Records the start of a new frame at the given timestamp (seconds).
#[allow(dead_code)]
pub fn render_stats_begin_frame(stats: &mut RenderStats, timestamp: f64) {
    stats.current_frame = FrameStats {
        begin_time: timestamp,
        end_time: timestamp,
        draw_calls: 0,
        triangle_count: 0,
    };
}

/// Finalizes the current frame at the given timestamp (seconds).
///
/// Moves `current_frame` into `last_frame` and increments `frame_count`.
#[allow(dead_code)]
pub fn render_stats_end_frame(stats: &mut RenderStats, timestamp: f64) {
    stats.current_frame.end_time = timestamp;
    stats.total_draw_calls = stats
        .total_draw_calls
        .saturating_add(stats.current_frame.draw_calls);
    stats.total_triangles = stats
        .total_triangles
        .saturating_add(stats.current_frame.triangle_count);
    stats.frame_count += 1;
    stats.last_frame = stats.current_frame.clone();
}

/// Records one draw call with the given triangle count for the current frame.
#[allow(dead_code)]
pub fn render_stats_add_draw_call(stats: &mut RenderStats, triangle_count: u32) {
    stats.current_frame.draw_calls += 1;
    stats.current_frame.triangle_count += u64::from(triangle_count);
}

/// Returns the last completed frame time in milliseconds.
#[allow(dead_code)]
pub fn render_stats_frame_time_ms(stats: &RenderStats) -> f64 {
    (stats.last_frame.end_time - stats.last_frame.begin_time) * 1000.0
}

/// Returns the estimated frames per second based on the last completed frame time.
///
/// Returns `0.0` if the frame time is zero or negative.
#[allow(dead_code)]
pub fn render_stats_fps(stats: &RenderStats) -> f64 {
    let dt = stats.last_frame.end_time - stats.last_frame.begin_time;
    if dt <= 0.0 {
        0.0
    } else {
        1.0 / dt
    }
}

/// Returns the draw call count recorded in the last completed frame.
#[allow(dead_code)]
pub fn render_stats_draw_calls(stats: &RenderStats) -> u32 {
    stats.last_frame.draw_calls
}

/// Returns the triangle count recorded in the last completed frame.
#[allow(dead_code)]
pub fn render_stats_triangle_count(stats: &RenderStats) -> u64 {
    stats.last_frame.triangle_count
}

/// Resets all cumulative counters and frame data to zero.
#[allow(dead_code)]
pub fn render_stats_reset(stats: &mut RenderStats) {
    stats.current_frame = FrameStats::default();
    stats.last_frame = FrameStats::default();
    stats.total_draw_calls = 0;
    stats.total_triangles = 0;
    stats.frame_count = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_render_stats_config();
        assert_eq!(cfg.history_capacity, 60);
        assert!(cfg.enable_history);
    }

    #[test]
    fn test_new_stats_zeroed() {
        let cfg = default_render_stats_config();
        let stats = new_render_stats(&cfg);
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.total_draw_calls, 0);
        assert_eq!(stats.total_triangles, 0);
    }

    #[test]
    fn test_begin_end_frame_increments_count() {
        let cfg = default_render_stats_config();
        let mut stats = new_render_stats(&cfg);
        render_stats_begin_frame(&mut stats, 0.0);
        render_stats_end_frame(&mut stats, 0.016);
        assert_eq!(stats.frame_count, 1);
    }

    #[test]
    fn test_frame_time_ms() {
        let cfg = default_render_stats_config();
        let mut stats = new_render_stats(&cfg);
        render_stats_begin_frame(&mut stats, 1.0);
        render_stats_end_frame(&mut stats, 1.016);
        let ft = render_stats_frame_time_ms(&stats);
        assert!((ft - 16.0).abs() < 0.01);
    }

    #[test]
    fn test_fps_calculation() {
        let cfg = default_render_stats_config();
        let mut stats = new_render_stats(&cfg);
        render_stats_begin_frame(&mut stats, 0.0);
        render_stats_end_frame(&mut stats, 0.016_666_67);
        let fps = render_stats_fps(&stats);
        assert!((fps - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_add_draw_call() {
        let cfg = default_render_stats_config();
        let mut stats = new_render_stats(&cfg);
        render_stats_begin_frame(&mut stats, 0.0);
        render_stats_add_draw_call(&mut stats, 1000);
        render_stats_add_draw_call(&mut stats, 500);
        render_stats_end_frame(&mut stats, 0.01);
        assert_eq!(render_stats_draw_calls(&stats), 2);
        assert_eq!(render_stats_triangle_count(&stats), 1500);
    }

    #[test]
    fn test_total_accumulation_over_frames() {
        let cfg = default_render_stats_config();
        let mut stats = new_render_stats(&cfg);
        for i in 0..3u64 {
            render_stats_begin_frame(&mut stats, i as f64 * 0.016);
            render_stats_add_draw_call(&mut stats, 100);
            render_stats_end_frame(&mut stats, i as f64 * 0.016 + 0.016);
        }
        assert_eq!(stats.total_draw_calls, 3);
        assert_eq!(stats.total_triangles, 300);
        assert_eq!(stats.frame_count, 3);
    }

    #[test]
    fn test_fps_zero_when_dt_zero() {
        let cfg = default_render_stats_config();
        let stats = new_render_stats(&cfg);
        assert_eq!(render_stats_fps(&stats), 0.0);
    }

    #[test]
    fn test_reset_clears_all() {
        let cfg = default_render_stats_config();
        let mut stats = new_render_stats(&cfg);
        render_stats_begin_frame(&mut stats, 0.0);
        render_stats_add_draw_call(&mut stats, 999);
        render_stats_end_frame(&mut stats, 0.1);
        render_stats_reset(&mut stats);
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.total_draw_calls, 0);
        assert_eq!(render_stats_frame_time_ms(&stats), 0.0);
    }

    #[test]
    fn test_draw_calls_reset_each_frame() {
        let cfg = default_render_stats_config();
        let mut stats = new_render_stats(&cfg);
        render_stats_begin_frame(&mut stats, 0.0);
        render_stats_add_draw_call(&mut stats, 50);
        render_stats_end_frame(&mut stats, 0.01);
        render_stats_begin_frame(&mut stats, 0.01);
        render_stats_end_frame(&mut stats, 0.02);
        assert_eq!(render_stats_draw_calls(&stats), 0);
    }
}
