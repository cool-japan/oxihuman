//! Performance statistics overlay data.

#[allow(dead_code)]
pub struct FrameStats {
    pub frame_number: u64,
    pub frame_time_ms: f32,
    pub fps: f32,
    pub draw_calls: u32,
    pub triangle_count: u32,
    pub vertex_count: u32,
    pub texture_memory_mb: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
}

#[allow(dead_code)]
pub struct StatsOverlay {
    pub history: Vec<FrameStats>,
    pub max_history: usize,
    pub visible: bool,
    pub show_graph: bool,
}

#[allow(dead_code)]
pub fn new_stats_overlay(max_history: usize) -> StatsOverlay {
    StatsOverlay {
        history: Vec::new(),
        max_history,
        visible: true,
        show_graph: false,
    }
}

#[allow(dead_code)]
pub fn push_frame_stats(overlay: &mut StatsOverlay, stats: FrameStats) {
    overlay.history.push(stats);
    while overlay.history.len() > overlay.max_history {
        overlay.history.remove(0);
    }
}

#[allow(dead_code)]
pub fn average_fps(overlay: &StatsOverlay) -> f32 {
    if overlay.history.is_empty() {
        return 0.0;
    }
    let sum: f32 = overlay.history.iter().map(|s| s.fps).sum();
    sum / overlay.history.len() as f32
}

#[allow(dead_code)]
pub fn average_frame_time(overlay: &StatsOverlay) -> f32 {
    if overlay.history.is_empty() {
        return 0.0;
    }
    let sum: f32 = overlay.history.iter().map(|s| s.frame_time_ms).sum();
    sum / overlay.history.len() as f32
}

#[allow(dead_code)]
pub fn peak_frame_time(overlay: &StatsOverlay) -> f32 {
    overlay
        .history
        .iter()
        .map(|s| s.frame_time_ms)
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn min_fps(overlay: &StatsOverlay) -> f32 {
    if overlay.history.is_empty() {
        return 0.0;
    }
    overlay
        .history
        .iter()
        .map(|s| s.fps)
        .fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn max_fps(overlay: &StatsOverlay) -> f32 {
    if overlay.history.is_empty() {
        return 0.0;
    }
    overlay
        .history
        .iter()
        .map(|s| s.fps)
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn average_triangle_count(overlay: &StatsOverlay) -> f32 {
    if overlay.history.is_empty() {
        return 0.0;
    }
    let sum: f32 = overlay
        .history
        .iter()
        .map(|s| s.triangle_count as f32)
        .sum();
    sum / overlay.history.len() as f32
}

#[allow(dead_code)]
pub fn average_draw_calls(overlay: &StatsOverlay) -> f32 {
    if overlay.history.is_empty() {
        return 0.0;
    }
    let sum: f32 = overlay.history.iter().map(|s| s.draw_calls as f32).sum();
    sum / overlay.history.len() as f32
}

#[allow(dead_code)]
pub fn format_stats_text(overlay: &StatsOverlay) -> String {
    let avg_fps = average_fps(overlay);
    let avg_ft = average_frame_time(overlay);
    let peak_ft = peak_frame_time(overlay);
    let mn_fps = min_fps(overlay);
    let mx_fps = max_fps(overlay);
    let avg_tris = average_triangle_count(overlay);
    let avg_dc = average_draw_calls(overlay);
    format!(
        "FPS: {:.1} (min {:.1} max {:.1})\nFrame time: {:.2}ms (peak {:.2}ms)\nTriangles: {:.0}\nDraw calls: {:.0}",
        avg_fps, mn_fps, mx_fps, avg_ft, peak_ft, avg_tris, avg_dc
    )
}

#[allow(dead_code)]
pub fn clear_history(overlay: &mut StatsOverlay) {
    overlay.history.clear();
}

#[allow(dead_code)]
pub fn history_len(overlay: &StatsOverlay) -> usize {
    overlay.history.len()
}

#[allow(dead_code)]
pub fn fps_from_frame_time(frame_time_ms: f32) -> f32 {
    if frame_time_ms <= 0.0 {
        return 0.0;
    }
    1000.0 / frame_time_ms
}

#[allow(dead_code)]
pub fn last_frame(overlay: &StatsOverlay) -> Option<&FrameStats> {
    overlay.history.last()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stats(frame: u64, fps: f32, ft_ms: f32, tris: u32, dc: u32) -> FrameStats {
        FrameStats {
            frame_number: frame,
            frame_time_ms: ft_ms,
            fps,
            draw_calls: dc,
            triangle_count: tris,
            vertex_count: tris * 3,
            texture_memory_mb: 64.0,
            cpu_time_ms: ft_ms * 0.4,
            gpu_time_ms: ft_ms * 0.6,
        }
    }

    #[test]
    fn test_new_stats_overlay() {
        let o = new_stats_overlay(60);
        assert_eq!(o.max_history, 60);
        assert!(o.history.is_empty());
        assert!(o.visible);
    }

    #[test]
    fn test_push_frame_stats() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 60.0, 16.67, 10000, 50));
        assert_eq!(history_len(&o), 1);
    }

    #[test]
    fn test_max_history_enforced() {
        let mut o = new_stats_overlay(5);
        for i in 0..10_u64 {
            push_frame_stats(&mut o, make_stats(i, 60.0, 16.67, 10000, 50));
        }
        assert_eq!(history_len(&o), 5);
    }

    #[test]
    fn test_average_fps() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 30.0, 33.33, 5000, 20));
        push_frame_stats(&mut o, make_stats(1, 60.0, 16.67, 10000, 40));
        let avg = average_fps(&o);
        assert!((avg - 45.0).abs() < 1e-4);
    }

    #[test]
    fn test_average_fps_empty() {
        let o = new_stats_overlay(60);
        assert!((average_fps(&o)).abs() < 1e-6);
    }

    #[test]
    fn test_peak_frame_time() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 60.0, 16.67, 0, 0));
        push_frame_stats(&mut o, make_stats(1, 20.0, 50.0, 0, 0));
        push_frame_stats(&mut o, make_stats(2, 60.0, 16.67, 0, 0));
        let peak = peak_frame_time(&o);
        assert!((peak - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_min_fps() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 60.0, 16.67, 0, 0));
        push_frame_stats(&mut o, make_stats(1, 30.0, 33.33, 0, 0));
        push_frame_stats(&mut o, make_stats(2, 90.0, 11.11, 0, 0));
        let mn = min_fps(&o);
        assert!((mn - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_max_fps() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 60.0, 16.67, 0, 0));
        push_frame_stats(&mut o, make_stats(1, 120.0, 8.33, 0, 0));
        let mx = max_fps(&o);
        assert!((mx - 120.0).abs() < 1e-3);
    }

    #[test]
    fn test_format_stats_text_non_empty() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 60.0, 16.67, 10000, 50));
        let text = format_stats_text(&o);
        assert!(!text.is_empty());
        assert!(text.contains("FPS"));
    }

    #[test]
    fn test_clear_history() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 60.0, 16.67, 0, 0));
        clear_history(&mut o);
        assert_eq!(history_len(&o), 0);
    }

    #[test]
    fn test_fps_from_frame_time() {
        let fps = fps_from_frame_time(16.666_667);
        assert!((fps - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_fps_from_frame_time_zero() {
        assert!((fps_from_frame_time(0.0)).abs() < 1e-6);
    }

    #[test]
    fn test_last_frame_none_when_empty() {
        let o = new_stats_overlay(60);
        assert!(last_frame(&o).is_none());
    }

    #[test]
    fn test_last_frame_some() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(42, 60.0, 16.67, 0, 0));
        let lf = last_frame(&o).expect("should succeed");
        assert_eq!(lf.frame_number, 42);
    }

    #[test]
    fn test_average_triangle_count_and_draw_calls() {
        let mut o = new_stats_overlay(60);
        push_frame_stats(&mut o, make_stats(0, 60.0, 16.67, 1000, 10));
        push_frame_stats(&mut o, make_stats(1, 60.0, 16.67, 3000, 30));
        assert!((average_triangle_count(&o) - 2000.0).abs() < 1.0);
        assert!((average_draw_calls(&o) - 20.0).abs() < 1.0);
    }
}
